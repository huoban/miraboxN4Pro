// 日志模块：内存环形日志（固定 50 行）+ 可选控制台镜像 + 推送到日志窗口
// GUI 模式无控制台；--console 时才输出到控制台

use std::collections::VecDeque;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Mutex, OnceLock};

const MAX_LINES: usize = 50;
const CONSOLE_QUEUE: usize = 256;

static CONSOLE_TX: Mutex<Option<SyncSender<String>>> = Mutex::new(None);
static UI_TX: Mutex<Option<SyncSender<String>>> = Mutex::new(None);
static HISTORY: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn history() -> &'static Mutex<VecDeque<String>> {
    HISTORY.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// 初始化日志；console=true 时同步输出到控制台（--console）
pub fn init(console: bool) {
    if !console {
        return;
    }
    let (tx, rx) = sync_channel::<String>(CONSOLE_QUEUE);
    std::thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            print!("{}", line);
        }
    });
    if let Ok(mut guard) = CONSOLE_TX.lock() {
        *guard = Some(tx);
    }
}

/// 注册日志窗口输出端（由 ui 模块调用）
pub fn set_ui_sender(tx: SyncSender<String>) {
    if let Ok(mut guard) = UI_TX.lock() {
        *guard = Some(tx);
    }
}

/// 取当前历史日志（窗口创建时装载）
pub fn history_snapshot() -> Vec<String> {
    history()
        .lock()
        .map(|guard| guard.iter().cloned().collect())
        .unwrap_or_default()
}

fn write_line(level: &str, msg: &str) {
    let line = format!("[{}] {} {}\n", crate::platform::win::now_text(), level, msg);

    if let Ok(mut guard) = history().lock() {
        guard.push_back(line.clone());
        while guard.len() > MAX_LINES {
            guard.pop_front();
        }
    }
    if let Ok(guard) = CONSOLE_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.try_send(line.clone());
        }
    }
    if let Ok(guard) = UI_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.try_send(line);
        }
    }
}

pub fn info(msg: impl AsRef<str>) {
    write_line("INFO ", msg.as_ref());
}

pub fn warn(msg: impl AsRef<str>) {
    write_line("WARN ", msg.as_ref());
}

pub fn error(msg: impl AsRef<str>) {
    write_line("ERROR", msg.as_ref());
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::logging::info(format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::logging::warn(format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::logging::error(format!($($arg)*)) };
}
