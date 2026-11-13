// N4 Pro 重制版入口
// 默认：控制台模式；-serve：静默后台（脱离控制台，仅写日志）

mod actions;
mod app;
mod config;
mod device;
mod logging;
mod platform;
mod render;

use platform::win::*;
use std::path::PathBuf;

fn arg_value(args: &[String], name: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().cloned();
        }
        if let Some(rest) = a.strip_prefix(&format!("{}=", name)) {
            return Some(rest.to_string());
        }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let serve = args.iter().any(|a| a.eq_ignore_ascii_case("-serve"));
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let dump_frames = args.iter().any(|a| a == "--dump-frames");
    let no_device = args.iter().any(|a| a == "--no-device");
    let once = args.iter().any(|a| a == "--once");
    let probe = args.iter().any(|a| a == "--probe");
    let log_input = args.iter().any(|a| a == "--log-input");
    let slot_test = args.iter().any(|a| a == "--slot-test");
    let no_bgpic = args.iter().any(|a| a == "--no-bgpic");
    let mode = arg_value(&args, "--mode");
    let profile = arg_value(&args, "--profile").map(PathBuf::from);

    let log_path = logging::init(!serve);
    // 崩溃信息写入日志，便于定位
    std::panic::set_hook(Box::new(|info| {
        crate::logging::error(format!("程序异常：{}", info));
    }));
    if serve {
        detach_console();
    }

    log_info!("N4ProCore 启动，版本 {}", env!("CARGO_PKG_VERSION"));
    log_info!("日志文件：{}", log_path.display());
    install_ctrl_handler();

    // 单实例互斥，避免双开抢占 HID
    let mutex_name = to_wide("Global\\N4ProCore_SingleInstance");
    let _mutex = unsafe { CreateMutexW(std::ptr::null_mut(), 0, mutex_name.as_ptr()) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        log_warn!("已有实例在运行，本次退出");
        if !serve {
            println!("已有实例在运行，本次退出");
        }
        return;
    }

    if serve {
        log_info!("静默后台模式");
    }
    if dry_run {
        log_info!("试运行模式：动作只记录不执行");
    }

    let mut cfg = match config::load(profile.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            log_error!("{}", e);
            if !serve {
                eprintln!("配置加载失败：{}", e);
            }
            return;
        }
    };
    if let Some(m) = mode {
        cfg.knob_mode = !m.eq_ignore_ascii_case("bar");
        log_info!("命令行覆盖触摸条模式：{}", if cfg.knob_mode { "旋钮" } else { "Control Bar" });
    }
    log_info!(
        "设备：{}，序列号：{}，场景：{}",
        cfg.device,
        cfg.serial,
        cfg.profile_dir.display()
    );
    log_info!(
        "亮度：{}，旋转：{}，触摸条模式：{}",
        cfg.brightness,
        cfg.rotation,
        if cfg.knob_mode { "旋钮" } else { "Control Bar" }
    );

    let mut app = match app::App::new(cfg, dry_run, dump_frames, no_device, once, probe, log_input, slot_test, no_bgpic) {
        Ok(a) => a,
        Err(e) => {
            log_error!("初始化失败：{}", e);
            if !serve {
                eprintln!("初始化失败：{}", e);
            }
            return;
        }
    };
    app.run();
    log_info!("N4ProCore 退出");
}
