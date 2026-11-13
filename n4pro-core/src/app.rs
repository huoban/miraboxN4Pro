// 应用核心：配置装载、页面渲染、HID 收发、输入事件、动作执行

use crate::actions::{self, Effect};
use crate::config::{self, ActionDef, Scene};
use crate::device::{input::InputEvent, protocol, HidDevice};
use crate::render::key::{render_key, render_label};
use crate::render::knob::render_knob;
use crate::render::ImageCache;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct App {
    cfg: config::AppConfig,
    scene: Scene,
    page_dir: PathBuf,
    page_idx: usize,
    actions: BTreeMap<String, ActionDef>,
    cache: ImageCache,
    knob_mode: bool,
    scroll: i32,
    tx: Option<Sender<Vec<u8>>>,
    dump: Option<std::fs::File>,
    dry_run: bool,
    key_down: [bool; 11],
    touch_active: bool,
    touch_start: Option<(i32, i32)>,
    touch_last: (i32, i32),
    last_bar_render: Instant,
    no_device: bool,
    once: bool,
    probe: bool,
    log_input: bool,
    slot_test: bool,
}

impl App {
    pub fn new(
        cfg: config::AppConfig,
        dry_run: bool,
        dump_frames: bool,
        no_device: bool,
        once: bool,
        probe: bool,
        log_input: bool,
        slot_test: bool,
    ) -> Result<Self, String> {
        let scene = config::load_scene(&cfg.profile_dir)?;
        let knob_mode = cfg.knob_mode;
        let dump = if dump_frames {
            let path = crate::platform::win::exe_dir().join("frames_dump.txt");
            crate::log_info!("帧转储文件：{}", path.display());
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
        } else {
            None
        };
        let mut app = App {
            cfg,
            scene,
            page_dir: PathBuf::new(),
            page_idx: 0,
            actions: BTreeMap::new(),
            cache: ImageCache::default(),
            knob_mode,
            scroll: 0,
            tx: None,
            dump,
            dry_run,
            key_down: [false; 11],
            touch_active: false,
            touch_start: None,
            touch_last: (0, 0),
            last_bar_render: Instant::now(),
            no_device,
            once,
            probe,
            log_input,
            slot_test,
        };
        app.resolve_current_page();
        app.load_current_page();
        Ok(app)
    }

    /// 根据 Pages.Current 定位当前页索引
    fn resolve_current_page(&mut self) {
        let current = self.scene.current_page.clone().unwrap_or_default();
        if let Some(pos) = self.scene.pages.iter().position(|p| *p == current) {
            self.page_idx = pos;
        } else {
            self.page_idx = 0;
        }
        crate::log_info!(
            "场景：{}，页面 {}/{}，当前页：{}",
            self.scene.name,
            self.page_idx + 1,
            self.scene.pages.len().max(1),
            current
        );
    }

    /// 加载当前页的动作与图片基准目录
    fn load_current_page(&mut self) {
        self.actions.clear();
        self.cache.clear();
        self.scroll = 0;
        let page_name = self.scene.pages.get(self.page_idx).cloned().unwrap_or_default();
        let scene_name = self
            .scene
            .dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if page_name == scene_name || page_name.is_empty() {
            self.page_dir = self.scene.dir.clone();
            self.actions = self.scene.root_actions.clone();
        } else {
            let dir = self.scene.dir.join("profiles").join(&page_name);
            match config::load_page_actions(&dir) {
                Ok(map) => {
                    self.page_dir = dir;
                    self.actions = map;
                }
                Err(e) => {
                    crate::log_error!("加载页面失败：{}", e);
                    self.page_dir = self.scene.dir.clone();
                    self.actions = self.scene.root_actions.clone();
                }
            }
        }
        crate::log_info!("当前页目录：{}，动作数：{}", self.page_dir.display(), self.actions.len());
    }

    pub fn run(&mut self) {
        // 打开设备
        let device = if self.no_device {
            crate::log_warn!("--no-device：跳过设备打开（离线验证）");
            None
        } else {
            match HidDevice::open_n4pro() {
                Ok(d) => {
                    crate::log_info!(
                        "已打开 N4 Pro：输出 {}B / 输入 {}B",
                        d.output_len,
                        d.input_len
                    );
                    Some(Arc::new(d))
                }
                Err(e) => {
                    crate::log_error!("设备打开失败：{}", e);
                    None
                }
            }
        };

        let mut reader_rx: Option<Receiver<Vec<u8>>> = None;
        if let Some(dev) = device {
            let input_len = dev.input_len.max(513);
            let (write_tx, write_rx) = channel::<Vec<u8>>();
            let (read_tx, read_rx) = channel::<Vec<u8>>();
            self.tx = Some(write_tx);
            reader_rx = Some(read_rx);

            let dev_writer = Arc::clone(&dev);
            std::thread::spawn(move || {
                while let Ok(frame) = write_rx.recv() {
                    if let Err(e) = dev_writer.write_frame(&frame) {
                        crate::log_error!("{}", e);
                        std::thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                    // 节流：避免瞬间高压导致设备丢帧/掉线
                    let tag = if frame.len() >= 9 { &frame[6..9] } else { &frame[..0] };
                    if tag == b"BAT" {
                        // 两张图片之间留出处理时间
                        std::thread::sleep(Duration::from_millis(60));
                    } else if tag == b"STP" || tag == b"CONNECT" {
                        std::thread::sleep(Duration::from_millis(30));
                    } else {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }
            });

            let dev_reader = Arc::clone(&dev);
            std::thread::spawn(move || {
                let mut buf = vec![0u8; input_len];
                loop {
                    match dev_reader.read_report(&mut buf) {
                        Ok(n) if n > 0 => {
                            if read_tx.send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            crate::log_error!("读取失败：{}", e);
                            std::thread::sleep(Duration::from_millis(500));
                        }
                    }
                }
            });

            // 等待设备就绪后再发命令
            std::thread::sleep(Duration::from_millis(300));

            if self.probe {
                crate::log_info!("--probe：仅发送 DIS + LIG 后退出");
                self.emit(vec![
                    protocol::dis(),
                    protocol::lig(self.cfg.brightness),
                ]);
                std::thread::sleep(Duration::from_millis(600));
                return;
            }

            // 初始化命令（与已验证可行的最小序列一致）
            self.emit(vec![
                protocol::dis(),
                protocol::lig(self.cfg.brightness),
            ]);

            // 2 秒心跳，避免固件超时断链
            if let Some(tx) = self.tx.clone() {
                std::thread::spawn(move || loop {
                    std::thread::sleep(Duration::from_secs(2));
                    if tx.send(protocol::rgbs()).is_err() {
                        break;
                    }
                });
            }
        } else {
            crate::log_warn!("无设备，进入离线模式（仅日志/帧转储）");
            if !self.no_device {
                crate::log_error!("设备不可用，程序退出");
                return;
            }
        }

        if self.slot_test {
            let mut frames = vec![protocol::cle()];
            for slot in 1u8..=15 {
                match render_label(&slot.to_string(), 112, 112, 48) {
                    Ok(png) => frames.extend(protocol::bat_frames(&png, slot)),
                    Err(e) => crate::log_error!("槽位 {} 标定图失败：{}", slot, e),
                }
            }
            frames.push(protocol::setlb());
            frames.push(protocol::stp());
            crate::log_info!("槽位标定模式：已发送 1~15 编号图");
            self.emit(frames);
        } else {
            self.refresh_all();
        }
        self.emit(vec![protocol::connect()]);

        if self.once {
            // 单次模式下短暂等待设备回执，便于确认命令被接收
            let deadline = Instant::now() + Duration::from_millis(1500);
            while Instant::now() < deadline {
                let Some(rx) = &reader_rx else { break };
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(buf) => self.handle_report(&buf),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            crate::log_info!("--once：完成一次整页刷新后退出");
            return;
        }

        // 主事件循环
        let rx = reader_rx;
        loop {
            match &rx {
                Some(rx) => match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(buf) => self.handle_report(&buf),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        crate::log_error!("HID 读线程已退出，程序结束");
                        break;
                    }
                },
                None => std::thread::sleep(Duration::from_millis(200)),
            }
        }
    }

    /// 发送帧（含转储）
    fn emit(&mut self, frames: Vec<Vec<u8>>) {
        if let Some(file) = self.dump.as_mut() {
            for frame in &frames {
                let hex: String = frame.iter().map(|b| format!("{:02x}", b)).collect();
                let _ = writeln!(file, "{}", hex);
            }
            let _ = file.flush();
        }
        if let Some(tx) = &self.tx {
            for frame in frames {
                let _ = tx.send(frame);
            }
        }
    }

    /// 整页刷新：与原厂实测一致（CLE → 10 键 BAT → 4 旋钮 BAT → SETLB → STP）
    /// 注意：BGPIC 背景层在旋钮模式下原厂不发送，发送后会导致按键不显示，故此处不发送。
    fn refresh_all(&mut self) {
        let mut frames = Vec::new();
        frames.push(protocol::cle());
        // 先发触摸条 4 个区域，再发 10 个按键，避免键图被覆盖
        for slot in 1u8..=4 {
            match self.knob_png(slot) {
                Ok(png) => frames.extend(protocol::bat_frames(&png, slot)),
                Err(e) => crate::log_error!("触摸条区域 {} 渲染失败：{}", slot, e),
            }
        }
        for key in 1u8..=10 {
            let slot = key_slot(key);
            let pos = key_pos(key);
            match self.key_png(&pos, key) {
                Ok(png) => frames.extend(protocol::bat_frames(&png, slot)),
                Err(e) => crate::log_error!("按键 {} 渲染失败：{}", key, e),
            }
        }
        frames.push(protocol::setlb());
        frames.push(protocol::stp());
        crate::log_info!("整页刷新完成，共 {} 帧", frames.len());
        self.emit(frames);
    }

    /// 旋钮图像（176×112）
    fn knob_png(&mut self, id: u8) -> Result<Vec<u8>, String> {
        let pos = format!("{},0", id.saturating_sub(1));
        let sig = format!("{}|knob|{}", self.page_dir.display(), pos);
        if let Some(v) = self.cache.get(&sig) {
            return Ok(v.clone());
        }
        let action = self.actions.get(&pos).cloned();
        let png = render_knob(action.as_ref(), &self.page_dir, 0)?;
        Ok(self.cache.put(sig, png))
    }

    fn key_png(&mut self, pos: &str, key: u8) -> Result<Vec<u8>, String> {
        let state_idx = if key > 0 && self.key_down[key as usize] && self.actions.get(pos).map(|a| a.states.len() > 1).unwrap_or(false) {
            1
        } else {
            0
        };
        let sig = format!("{}|{}|{}", self.page_dir.display(), pos, state_idx);
        if let Some(v) = self.cache.get(&sig) {
            return Ok(v.clone());
        }
        let action = self.actions.get(pos).cloned();
        let png = render_key(action.as_ref(), &self.page_dir, state_idx)?;
        Ok(self.cache.put(sig, png))
    }

    /// 处理一帧输入报告
    fn handle_report(&mut self, buf: &[u8]) {
        if self.log_input {
            let head: String = buf.iter().take(24).map(|b| format!("{:02x}", b)).collect();
            crate::log_info!("输入 {} 字节：{}", buf.len(), head);
        }
        let Some(event) = crate::device::input::parse(buf) else {
            return;
        };
        match event {
            InputEvent::Key { index, down } => self.on_key(index, down),
            InputEvent::DialRotate { id } => self.on_dial_rotate(id),
            InputEvent::DialPress { id, down } => {
                if down {
                    self.on_dial_press(id);
                }
            }
            InputEvent::TouchDown => {
                self.touch_active = true;
                self.touch_start = None;
                self.touch_last = (0, 0);
            }
            InputEvent::TouchMove { x, y } => {
                if self.touch_active {
                    if self.touch_start.is_none() {
                        self.touch_start = Some((x, y));
                    }
                    self.touch_last = (x, y);
                    self.on_touch_drag();
                }
            }
            InputEvent::TouchUp | InputEvent::GestureEnd => self.finish_touch(),
            InputEvent::ModeChanged { control_bar } => {
                self.knob_mode = !control_bar;
                crate::log_info!(
                    "固件切换模式：{}",
                    if control_bar { "Control Bar" } else { "旋钮模式" }
                );
                self.cache.clear();
                self.refresh_all();
            }
            InputEvent::Unknown { index, state } => {
                crate::log_info!("未知输入事件：idx=0x{:02X} state={}", index, state);
            }
        }
    }

    fn on_key(&mut self, index: u8, down: bool) {
        if index == 0 || index > 10 {
            return;
        }
        self.key_down[index as usize] = down;
        let pos = key_pos(index);
        if down {
            if let Some(def) = self.actions.get(&pos).cloned() {
                let effect = actions::execute(&def, self.dry_run, 0);
                self.apply_effect(effect);
            } else {
                crate::log_info!("按键 {}（{}）未绑定动作", index, pos);
            }
        }
        // 更新该键图像
        if let Ok(png) = self.key_png(&pos, index) {
            let slot = key_slot(index);
            self.emit(protocol::bat_frames(&png, slot));
        }
    }

    fn on_dial_rotate(&mut self, id: u8) {
        let pos = format!("{},0", id.saturating_sub(1));
        if let Some(def) = self.actions.get(&pos).cloned() {
            crate::log_info!("旋钮 {} 旋转，执行 {}", id, def.name);
            let effect = actions::execute(&def, self.dry_run, 0);
            self.apply_effect(effect);
        } else {
            crate::log_info!("旋钮 {} 旋转（未绑定）", id);
        }
    }

    fn on_dial_press(&mut self, id: u8) {
        let pos = format!("{},0", id.saturating_sub(1));
        if let Some(def) = self.actions.get(&pos).cloned() {
            crate::log_info!("旋钮 {} 按下，执行 {}", id, def.name);
            let effect = actions::execute(&def, self.dry_run, 0);
            self.apply_effect(effect);
        }
    }

    fn on_touch_drag(&mut self) {
        if self.knob_mode {
            return;
        }
        let Some((sx, _)) = self.touch_start else { return };
        let max_scroll = (self.scene.touchbar.len() as i32 * 160 - 800).max(0);
        let offset = (sx - self.touch_last.0).clamp(0, max_scroll);
        self.scroll = offset;
    }

    fn finish_touch(&mut self) {
        if !self.touch_active {
            return;
        }
        self.touch_active = false;
        let Some((sx, sy)) = self.touch_start else { return };
        let dx = self.touch_last.0 - sx;
        let dy = self.touch_last.1 - sy;
        if dx.abs() > 60 && dy.abs() < 45 {
            // 坐标方向与面板一致：x 增大表示向左滑
            if dx > 0 {
                self.switch_page(1);
            } else {
                self.switch_page(-1);
            }
        } else if dy.abs() > 35 && dx.abs() < 60 {
            // 上下滑动由固件上报 0xB1/0xB2 处理，这里不再二次判定，避免重复切换
        }
    }

    fn switch_page(&mut self, delta: i32) {
        if self.scene.pages.is_empty() {
            return;
        }
        let len = self.scene.pages.len() as i32;
        let next = ((self.page_idx as i32 + delta) % len + len) % len;
        self.page_idx = next as usize;
        crate::log_info!("切换到页面 {}/{}", self.page_idx + 1, len);
        self.load_current_page();
        self.refresh_all();
    }

    fn apply_effect(&mut self, effect: Effect) {
        match effect {
            Effect::None => {}
            Effect::NextPage => self.switch_page(1),
            Effect::PrevPage => self.switch_page(-1),
            Effect::SwitchScene(uuid) => self.switch_scene(&uuid),
        }
    }

    fn switch_scene(&mut self, uuid: &str) {
        let Some(parent) = self.scene.dir.parent() else {
            crate::log_warn!("无法定位场景根目录");
            return;
        };
        let target = parent.join(uuid);
        if !target.join("manifest.json").exists() {
            crate::log_warn!("目标场景不存在：{}", target.display());
            return;
        }
        match config::load_scene(&target) {
            Ok(scene) => {
                crate::log_info!("切换场景：{}", scene.name);
                self.scene = scene;
                self.resolve_current_page();
                self.load_current_page();
                self.cache.clear();
                self.refresh_all();
            }
            Err(e) => crate::log_error!("切换场景失败：{}", e),
        }
    }
}

/// 按键编号（1..10）→ 配置位置
pub fn key_pos(key: u8) -> String {
    if key == 0 {
        return "0,0".to_string();
    }
    let x = (key - 1) % 5;
    let y = if key <= 5 { 1 } else { 2 };
    format!("{},{}", x, y)
}

/// 按键编号（1..10）→ 设备图像槽位（按用户实测标定）
/// 上排 1~5 → 6~10；下排 6~10 → 11~15（即 slot = key + 5）
pub fn key_slot(key: u8) -> u8 {
    key.saturating_add(5).min(15)
}
