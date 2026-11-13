// 应用核心：配置装载、页面渲染、HID 收发、输入事件、动作执行

use crate::actions::{self, Effect, Trigger};
use crate::config::{self, ActionDef, Scene};
use crate::device::{input::InputEvent, protocol, HidDevice};
use crate::render::key::{render_key, render_label};
use crate::render::knob::render_knob;
use crate::render::touchbar::{render_black_bar, render_control_bar, BTN_STEP};
use crate::render::ImageCache;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 写线程消息：整页刷新批次可被更新的一批整体替换，避免队列积压
enum WriteMsg {
    /// 普通帧（心跳、触摸条局部刷新等）
    Frames(Vec<Vec<u8>>),
    /// 整页刷新批次（以 CLE 开头）
    Refresh(Vec<Vec<u8>>),
}

/// 会话结束方式：退出程序 / 等待设备后重连
enum SessionEnd {
    Exit,
    Reconnect,
}

/// 退出信号句柄：供控制台 Ctrl+C/关窗回调直接写结束帧
static EXIT_DEV: Mutex<Option<Arc<HidDevice>>> = Mutex::new(None);

/// 会话运行标记：心跳线程据此退出
static SESSION_RUNNING: AtomicBool = AtomicBool::new(false);

/// 退出标记：置位后写线程停止发送图像帧，保证结束帧是最后一帧
static EXITING: AtomicBool = AtomicBool::new(false);

/// 控制台退出回调：先停写线程，再直接同步写 CLE+DC 帧，避免传输中途断开
pub fn send_exit_frame_from_handler() {
    let dev = EXIT_DEV.lock().ok().and_then(|guard| guard.clone());
    if let Some(dev) = dev {
        EXITING.store(true, Ordering::Relaxed);
        // 等待当前帧写完，避免中断图像传输
        std::thread::sleep(Duration::from_millis(100));
        match dev.write_frame(&protocol::exit_frames()) {
            Ok(()) => crate::log_info!("已发送退出信号（CLE+DC）"),
            Err(e) => crate::log_warn!("退出信号发送失败：{}", e),
        }
        // 给设备固件留出处理时间
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// 逐帧写入设备，并对图像头/结束帧做少量节流
fn write_frames(dev: &HidDevice, frames: &[Vec<u8>]) {
    for frame in frames {
        if EXITING.load(Ordering::Relaxed) {
            return;
        }
        if let Err(e) = dev.write_frame(frame) {
            crate::log_error!("{}", e);
            std::thread::sleep(Duration::from_millis(200));
            continue;
        }
        let tag = if frame.len() >= 9 { &frame[6..9] } else { &frame[..0] };
        if tag == b"BAT" {
            std::thread::sleep(Duration::from_millis(10));
        } else if tag == b"STP" || tag == b"CONNECT" {
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

pub struct App {
    cfg: config::AppConfig,
    scene: Scene,
    page_dir: PathBuf,
    page_idx: usize,
    actions: BTreeMap<String, ActionDef>,
    cache: ImageCache,
    knob_mode: bool,
    scroll: i32,
    scroll_start: i32,
    no_bgpic: bool,
    /// 设备是否处于息屏状态
    sleeping: bool,
    tx: Option<Sender<WriteMsg>>,
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
        no_bgpic: bool,
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
            scroll_start: 0,
            no_bgpic,
            sleeping: false,
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
        // 官方把触摸条滚动位置存在 Settings.widgetCurrX（负值），启动时恢复
        let max = app.max_scroll();
        app.scroll = (-app.scene.touchbar_curr_x).clamp(0, max);
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
        // 外层循环：设备断开后关闭句柄、等待 USB 接入、重新初始化
        loop {
            match self.run_session() {
                SessionEnd::Exit => return,
                SessionEnd::Reconnect => {
                    crate::log_warn!("设备通信中断，进入 USB 等待模式");
                    crate::platform::device_watch::wait_for_device();
                }
            }
        }
    }

    /// 单次设备会话；返回 Exit（不再重连）或 Reconnect（等待设备）
    fn run_session(&mut self) -> SessionEnd {
        if self.no_device {
            crate::log_warn!("--no-device：跳过设备打开（离线验证）");
            return self.run_offline();
        }

        let dev = match HidDevice::open_n4pro() {
            Ok(d) => {
                crate::log_info!(
                    "已打开 N4 Pro：输出 {}B / 输入 {}B",
                    d.output_len,
                    d.input_len
                );
                Arc::new(d)
            }
            Err(e) => {
                crate::log_warn!("设备打开失败：{}", e);
                return SessionEnd::Reconnect;
            }
        };

        let input_len = dev.input_len.max(513);
        let (write_tx, write_rx) = channel::<WriteMsg>();
        let (read_tx, read_rx) = channel::<Vec<u8>>();
        self.tx = Some(write_tx.clone());
        if let Ok(mut guard) = EXIT_DEV.lock() {
            *guard = Some(Arc::clone(&dev));
        }
        SESSION_RUNNING.store(true, Ordering::SeqCst);

        let dev_writer = Arc::clone(&dev);
        std::thread::spawn(move || {
            while let Ok(msg) = write_rx.recv() {
                match msg {
                    // 整页刷新：丢弃排队中的旧批次/旧帧，只发送最新一批，避免积压
                    WriteMsg::Refresh(frames) => {
                        let mut latest = frames;
                        let mut tail: Vec<WriteMsg> = Vec::new();
                        while let Ok(next) = write_rx.try_recv() {
                            match next {
                                WriteMsg::Refresh(f) => {
                                    latest = f;
                                    tail.clear();
                                }
                                other => tail.push(other),
                            }
                        }
                        write_frames(&dev_writer, &latest);
                        // 保留刷新批次之后的消息（如启动握手 CONNECT）
                        for msg in tail {
                            if let WriteMsg::Frames(f) = msg {
                                write_frames(&dev_writer, &f);
                            }
                        }
                    }
                    WriteMsg::Frames(frames) => write_frames(&dev_writer, &frames),
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
                        // 空帧作为断开标记，通知主循环结束本次会话
                        let _ = read_tx.send(Vec::new());
                        break;
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
            send_exit_frame_from_handler();
            self.end_session();
            return SessionEnd::Exit;
        }

        // 初始化命令（与已验证可行的最小序列一致）
        self.emit(vec![
            protocol::dis(),
            protocol::lig(self.cfg.brightness),
        ]);

        // 2 秒心跳，避免固件超时断链；会话结束后自动退出
        if let Some(tx) = self.tx.clone() {
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(2));
                if !SESSION_RUNNING.load(Ordering::SeqCst) {
                    break;
                }
                if tx.send(WriteMsg::Frames(vec![protocol::rgbs()])).is_err() {
                    break;
                }
            });
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
                match read_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(buf) if !buf.is_empty() => self.handle_report(&buf),
                    Ok(_) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            crate::log_info!("--once：完成一次整页刷新后退出");
            send_exit_frame_from_handler();
            self.end_session();
            return SessionEnd::Exit;
        }

        // 主事件循环：空帧=设备断开
        loop {
            match read_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(buf) if buf.is_empty() => {
                    crate::log_warn!("HID 读取线程检测到设备断开");
                    break;
                }
                Ok(buf) => self.handle_report(&buf),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    crate::log_warn!("HID 读线程已退出");
                    break;
                }
            }
        }
        self.end_session();
        SessionEnd::Reconnect
    }

    /// 离线模式（--no-device）：仅渲染与日志
    fn run_offline(&mut self) -> SessionEnd {
        if self.slot_test {
            let mut frames = vec![protocol::cle()];
            for slot in 1u8..=15 {
                if let Ok(png) = render_label(&slot.to_string(), 112, 112, 48) {
                    frames.extend(protocol::bat_frames(&png, slot));
                }
            }
            frames.push(protocol::setlb());
            frames.push(protocol::stp());
            self.emit(frames);
        } else {
            self.refresh_all();
        }
        if self.once {
            crate::log_info!("--once：完成一次整页刷新后退出");
            return SessionEnd::Exit;
        }
        loop {
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// 结束会话：停止写线程、释放设备句柄
    fn end_session(&mut self) {
        SESSION_RUNNING.store(false, Ordering::SeqCst);
        self.tx = None;
        if let Ok(mut guard) = EXIT_DEV.lock() {
            *guard = None;
        }
        // 等待写线程处理完当前帧并退出
        std::thread::sleep(Duration::from_millis(200));
    }

    /// 发送普通帧（含转储）
    fn emit(&mut self, frames: Vec<Vec<u8>>) {
        self.emit_msg(WriteMsg::Frames(frames));
    }

    /// 发送整页刷新批次（写线程会合并掉过期的旧批次）
    fn emit_refresh(&mut self, frames: Vec<Vec<u8>>) {
        self.emit_msg(WriteMsg::Refresh(frames));
    }

    fn emit_msg(&mut self, msg: WriteMsg) {
        let frames = match &msg {
            WriteMsg::Frames(f) | WriteMsg::Refresh(f) => f,
        };
        if let Some(file) = self.dump.as_mut() {
            for frame in frames {
                let hex: String = frame.iter().map(|b| format!("{:02x}", b)).collect();
                let _ = writeln!(file, "{}", hex);
            }
            let _ = file.flush();
        }
        if let Some(tx) = &self.tx {
            let _ = tx.send(msg);
        }
    }

    /// 触摸条帧（Control Bar=BGPIC / 旋钮模式=黑条+1~4 区）
    fn touchbar_frames(&mut self) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        if self.knob_mode {
            if !self.no_bgpic {
                if let Ok(black) = render_black_bar() {
                    frames.extend(protocol::bgpic_frames(&black, 800, 130));
                }
            }
            for slot in 1u8..=4 {
                match self.knob_png(slot) {
                    Ok(png) => frames.extend(protocol::bat_frames(&png, slot)),
                    Err(e) => crate::log_error!("触摸条区域 {} 渲染失败：{}", slot, e),
                }
            }
        } else if !self.no_bgpic {
            match self.bar_png() {
                Ok(png) => frames.extend(protocol::bgpic_frames(&png, 800, 130)),
                Err(e) => crate::log_error!("Control Bar 渲染失败：{}", e),
            }
        }
        frames
    }

    /// 只刷新触摸条（模式切换 / 滑动时使用，不动按键）
    fn refresh_touchbar(&mut self) {
        let mut frames = self.touchbar_frames();
        frames.push(protocol::setlb());
        frames.push(protocol::stp());
        crate::log_info!("触摸条刷新，共 {} 帧", frames.len());
        self.emit(frames);
    }

    /// 整页刷新：CLE → 触摸条 → 10 键 BAT → SETLB → STP（仅切页/启动时调用）
    fn refresh_all(&mut self) {
        let mut frames = vec![protocol::cle()];
        frames.extend(self.touchbar_frames());

        // 10 个物理按键（映射保持 slot = key + 5，不再改动）
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
        crate::log_info!(
            "整页刷新完成，共 {} 帧（{}）",
            frames.len(),
            if self.knob_mode { "旋钮模式" } else { "Control Bar" }
        );
        self.emit_refresh(frames);
    }

    /// Control Bar 最大滚动偏移：保证最后一个按钮能完整显示
    fn max_scroll(&self) -> i32 {
        let n = self.scene.touchbar.len() as i32;
        if n <= 0 {
            return 0;
        }
        let content_right = 25 + (n - 1) * BTN_STEP + 142;
        (content_right - 780).max(0)
    }

    /// Control Bar 整条图像（800×130）
    fn bar_png(&mut self) -> Result<Vec<u8>, String> {
        let max = self.max_scroll();
        let sig = format!(
            "bar|{}|{}|{}",
            self.scene.dir.display(),
            self.scroll,
            self.page_idx
        );
        if let Some(v) = self.cache.get(&sig) {
            return Ok(v.clone());
        }
        let png = render_control_bar(
            &self.scene.touchbar,
            &self.scene.dir,
            self.scroll,
            self.scroll > 0,
            self.scroll < max,
        )?;
        Ok(self.cache.put(sig, png))
    }

    /// 旋钮图像（176×112），页码用于翻页动作的内置图
    fn knob_png(&mut self, id: u8) -> Result<Vec<u8>, String> {
        let pos = format!("{},0", id.saturating_sub(1));
        let page_number = (self.page_idx + 1).min(10) as u8;
        let sig = format!("{}|knob|{}|{}", self.page_dir.display(), pos, page_number);
        if let Some(v) = self.cache.get(&sig) {
            return Ok(v.clone());
        }
        let action = self.actions.get(&pos).cloned();
        let png = render_knob(action.as_ref(), &self.page_dir, 0, page_number)?;
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
            InputEvent::DialRotate { id, dir } => self.on_dial_rotate(id, dir),
            InputEvent::DialPress { id, down } => {
                if down {
                    self.on_dial_press(id);
                }
            }
            InputEvent::TouchDown => {
                self.touch_active = true;
                self.touch_start = None;
                self.touch_last = (0, 0);
                self.scroll_start = self.scroll;
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
                self.refresh_touchbar();
            }
            InputEvent::BarTap { code, x } => self.on_bar_tap(code, x),
            InputEvent::Unknown { index, state } => {
                crate::log_info!("未知输入事件：idx=0x{:02X} state={}", index, state);
            }
        }
    }

    /// Control Bar 点击命中：index = (X + scroll - 25) / 152
    fn on_bar_tap(&mut self, code: u8, x: i32) {
        if self.knob_mode {
            return;
        }
        if code == 0x39 {
            crate::log_info!("Control Bar 返回键（0x39），忽略");
            return;
        }
        let count = self.scene.touchbar.len() as i32;
        if count == 0 {
            return;
        }
        let idx = if x >= 0 {
            (x + self.scroll - 25) / BTN_STEP
        } else {
            code as i32 - 0x40
        };
        if idx < 0 || idx >= count {
            crate::log_info!("Control Bar 点击越界：code=0x{:02X} x={} idx={}", code, x, idx);
            return;
        }
        if let Some(def) = self.scene.touchbar.get(idx as usize).cloned() {
            crate::log_info!(
                "Control Bar 点击：code=0x{:02X} x={} -> 第 {} 项（{}）",
                code,
                x,
                idx + 1,
                def.name
            );
            let effect = actions::execute(&def, self.dry_run, 0, Trigger::Press);
            self.apply_effect(effect);
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
                let effect = actions::execute(&def, self.dry_run, 0, Trigger::Press);
                self.apply_effect(effect);
            } else {
                crate::log_info!("按键 {}（{}）未绑定动作", index, pos);
            }
        }
        // 按用户要求：按键操作不触发图像刷新，仅在切页时整页重绘
    }

    fn on_dial_rotate(&mut self, id: u8, dir: i8) {
        let pos = format!("{},0", id.saturating_sub(1));
        if let Some(def) = self.actions.get(&pos).cloned() {
            crate::log_info!(
                "旋钮 {} 旋转（{}），执行 {}",
                id,
                if dir > 0 { "顺时针" } else { "逆时针" },
                def.name
            );
            if def.uuid.to_lowercase().contains("profile.rotate") {
                // 官方在 ProfileUUID 为空时按方向循环切换场景
                self.cycle_scene(if dir > 0 { 1 } else { -1 });
                return;
            }
            let effect = actions::execute(&def, self.dry_run, 0, Trigger::Rotate(dir));
            self.apply_effect(effect);
        } else {
            crate::log_info!("旋钮 {} 旋转（未绑定）", id);
        }
    }

    fn on_dial_press(&mut self, id: u8) {
        let pos = format!("{},0", id.saturating_sub(1));
        if let Some(def) = self.actions.get(&pos).cloned() {
            crate::log_info!("旋钮 {} 按下，执行 {}", id, def.name);
            if def.uuid.to_lowercase().contains("profile.rotate") {
                self.cycle_scene(1);
                return;
            }
            let effect = actions::execute(&def, self.dry_run, 0, Trigger::Press);
            self.apply_effect(effect);
        }
    }

    /// 循环切换场景目录（*.sdProfile），dir=1 下一个、-1 上一个
    fn cycle_scene(&mut self, dir: i8) {
        let Some(parent) = self.cfg.profile_dir.parent().map(|p| p.to_path_buf()) else {
            return;
        };
        let Ok(rd) = std::fs::read_dir(&parent) else { return };
        let mut scenes: Vec<PathBuf> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.is_dir()
                    && p.extension()
                        .map(|x| x.eq_ignore_ascii_case("sdProfile"))
                        .unwrap_or(false)
            })
            .collect();
        if scenes.is_empty() {
            return;
        }
        scenes.sort();
        let cur = scenes
            .iter()
            .position(|p| p == &self.cfg.profile_dir)
            .unwrap_or(0) as i32;
        let next = (cur + dir as i32).rem_euclid(scenes.len() as i32) as usize;
        let path = scenes[next].clone();
        match config::load_scene(&path) {
            Ok(scene) => {
                self.scene = scene;
                self.cfg.profile_dir = path;
                self.page_idx = 0;
                self.resolve_current_page();
                self.load_current_page();
                crate::log_info!("场景切换：{}", self.scene.name);
                self.refresh_all();
            }
            Err(e) => crate::log_warn!("场景切换失败：{}", e),
        }
    }

    fn on_touch_drag(&mut self) {
        if self.knob_mode {
            return;
        }
        let Some((sx, _)) = self.touch_start else { return };
        let max = self.max_scroll();
        let offset = (self.scroll_start + (sx - self.touch_last.0)).clamp(0, max);
        if offset != self.scroll {
            self.scroll = offset;
            // 跟手刷新（约 60ms 节流）
            if self.last_bar_render.elapsed() > Duration::from_millis(60) && !self.no_bgpic {
                self.last_bar_render = Instant::now();
                if let Ok(png) = self.bar_png() {
                    self.emit(protocol::bgpic_frames(&png, 800, 130));
                }
            }
        }
    }

    fn finish_touch(&mut self) {
        if !self.touch_active {
            return;
        }
        self.touch_active = false;
        let Some((sx, sy)) = self.touch_start else { return };
        let dx = self.touch_last.0 - sx;
        let dy = self.touch_last.1 - sy;

        if !self.knob_mode {
            // Control Bar：横滑滚动按钮条，抬手吸附到 152px 网格
            let max = self.max_scroll();
            let snapped =
                (((self.scroll as f64) / (BTN_STEP as f64)).round() as i32 * BTN_STEP).clamp(0, max);
            if snapped != self.scroll {
                self.scroll = snapped;
                if !self.no_bgpic {
                    if let Ok(png) = self.bar_png() {
                        self.emit(protocol::bgpic_frames(&png, 800, 130));
                    }
                }
            }
            return;
        }

        if dx.abs() > 60 && dy.abs() < 45 {
            // 旋钮模式：坐标方向与面板一致，x 增大表示向左滑
            if dx > 0 {
                self.switch_page(1);
            } else {
                self.switch_page(-1);
            }
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
            Effect::GotoPage(target) => self.goto_page(&target),
            Effect::BackToParent => self.back_to_parent(),
            Effect::SwitchScene(uuid) => self.switch_scene(&uuid),
            Effect::BrightnessDelta(delta) => self.adjust_brightness(delta),
            Effect::DeviceSleepToggle => self.toggle_device_sleep(),
        }
    }

    /// 跳转到指定页（页码或页面 UUID）
    fn goto_page(&mut self, target: &str) {
        if self.scene.pages.is_empty() {
            crate::log_warn!("场景没有页面，忽略跳转");
            return;
        }
        let idx = if let Ok(n) = target.trim().parse::<usize>() {
            // 页码从 1 开始；超出范围时取最后一页
            (n.saturating_sub(1)).min(self.scene.pages.len() - 1)
        } else if let Some(pos) = self.scene.pages.iter().position(|p| p == target) {
            pos
        } else {
            crate::log_warn!("未找到目标页面：{}", target);
            return;
        };
        self.page_idx = idx;
        crate::log_info!(
            "切换到页面 {}/{}",
            self.page_idx + 1,
            self.scene.pages.len()
        );
        self.load_current_page();
        self.refresh_all();
    }

    /// 返回场景根页
    fn back_to_parent(&mut self) {
        let scene_name = self
            .scene
            .dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if let Some(root) = self.scene.pages.iter().position(|p| *p == scene_name) {
            if root != self.page_idx {
                self.page_idx = root;
                crate::log_info!("返回场景根页 {}/{}", root + 1, self.scene.pages.len());
                self.load_current_page();
                self.refresh_all();
                return;
            }
        }
        crate::log_info!("已在场景根页，忽略返回动作");
    }

    /// 调整设备亮度（增量）
    fn adjust_brightness(&mut self, delta: i32) {
        let next = (self.cfg.brightness as i32 + delta).clamp(0, 100) as u8;
        if next == self.cfg.brightness && !self.sleeping {
            return;
        }
        self.cfg.brightness = next;
        self.sleeping = false;
        self.emit(vec![protocol::lig(next)]);
        crate::log_info!("设备亮度调整为 {}", next);
    }

    /// 设备息屏切换（亮度 0 / 恢复）
    fn toggle_device_sleep(&mut self) {
        self.sleeping = !self.sleeping;
        let level = if self.sleeping { 0 } else { self.cfg.brightness };
        self.emit(vec![protocol::lig(level)]);
        crate::log_info!("设备息屏：{}", self.sleeping);
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
