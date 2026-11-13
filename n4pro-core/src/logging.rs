// 日志模块：仅控制台输出（独立线程 + 有界队列），不写文件
// 控制台缓冲区由 win::setup_console 固定为 50 行，旧日志自动滚掉

use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Mutex;

/// 控制台待输出队列上限：写满直接丢弃，保证业务线程永不阻塞
const CONSOLE_QUEUE: usize = 256;

static CONSOLE_TX: Mutex<Option<SyncSender<String>>> = Mutex::new(None);

/// 初始化控制台日志；console=false（-serve）时不输出任何日志
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

fn write_line(level: &str, msg: &str) {
    let line = format!("[{}] {} {}\n", crate::platform::win::now_text(), level, msg);
    // 只入队，不直接写控制台：队列满即丢弃，避免控制台选中文本/管道堵塞卡住业务线程
    if let Ok(guard) = CONSOLE_TX.lock() {
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
