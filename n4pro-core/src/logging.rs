// 日志模块：写入 exe 运行目录 logs\N4Pro.log，单文件 2MB，保留 3 份滚动
// 控制台模式由独立线程输出（有界队列），避免控制台选中文本/管道阻塞主线程；-serve 模式仅写文件。

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Mutex;

const MAX_SIZE: u64 = 2 * 1024 * 1024;
/// 控制台待输出队列上限：写满直接丢弃，保证业务线程永不阻塞
const CONSOLE_QUEUE: usize = 256;

static CONSOLE_TX: Mutex<Option<SyncSender<String>>> = Mutex::new(None);
static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

struct Logger {
    path: PathBuf,
    file: File,
}

fn base_dir() -> PathBuf {
    crate::platform::win::exe_dir()
}

/// 初始化日志；返回实际使用的日志文件路径
pub fn init(console: bool) -> PathBuf {
    if console {
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
    let mut dir = base_dir().join("logs");
    if fs::create_dir_all(&dir).is_err() {
        dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("logs");
        let _ = fs::create_dir_all(&dir);
    }
    let path = dir.join("N4Pro.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .or_else(|_| {
            let fallback = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("N4Pro.log");
            OpenOptions::new().create(true).append(true).open(fallback)
        });
    if let Ok(f) = file {
        *LOGGER.lock().unwrap() = Some(Logger { path: path.clone(), file: f });
    }
    path
}

fn rotate_if_needed(logger: &mut Logger) {
    let size = logger.file.metadata().map(|m| m.len()).unwrap_or(0);
    if size < MAX_SIZE {
        return;
    }
    let dir = logger.path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let base = logger.path.file_stem().and_then(|s| s.to_str()).unwrap_or("n4pro");
    let ext = logger.path.extension().and_then(|s| s.to_str()).unwrap_or("log");
    let old2 = dir.join(format!("{}.2.{}", base, ext));
    let old1 = dir.join(format!("{}.1.{}", base, ext));
    let _ = fs::remove_file(&old2);
    let _ = fs::rename(&old1, &old2);
    let _ = fs::rename(&logger.path, &old1);
    if let Ok(f) = OpenOptions::new().create(true).append(true).open(&logger.path) {
        logger.file = f;
    }
}

fn write_line(level: &str, msg: &str) {
    let line = format!("[{}] {} {}\n", crate::platform::win::now_text(), level, msg);
    // 控制台输出只入队，不直接写控制台：队列满即丢弃，避免选中文本/管道堵塞卡住业务线程
    if let Ok(guard) = CONSOLE_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.try_send(line.clone());
        }
    }
    if let Ok(mut guard) = LOGGER.lock() {
        if let Some(logger) = guard.as_mut() {
            let _ = logger.file.write_all(line.as_bytes());
            let _ = logger.file.flush();
            rotate_if_needed(logger);
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
