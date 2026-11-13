// 音频播放器（Soundboard）：MCI 播放本地音频文件，支持音量/循环/淡入，停止全部
// 官方设置键：actionType、fadeType、fadeDuration、outputType、volume + 音频文件路径

use crate::actions::{s_bool, s_i64, s_str};
use crate::platform::win::to_wide;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[link(name = "winmm")]
extern "system" {
    fn mciSendStringW(cmd: *const u16, ret: *mut u16, ret_len: u32, hwnd: isize) -> u32;
}

/// 正在播放的 MCI 别名
static ALIASES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static ALIAS_SEQ: AtomicUsize = AtomicUsize::new(0);

fn aliases() -> &'static Mutex<Vec<String>> {
    ALIASES.get_or_init(|| Mutex::new(Vec::new()))
}

fn mci(cmd: &str) -> u32 {
    let wide = to_wide(cmd);
    unsafe { mciSendStringW(wide.as_ptr(), std::ptr::null_mut(), 0, 0) }
}

fn set_volume(alias: &str, volume: i64) {
    let v = volume.clamp(0, 100) * 10;
    mci(&format!("setaudio {} volume to {}", alias, v));
}

/// 播放音频
pub fn play(settings: &Value, dry_run: bool) {
    let path = s_str(
        settings,
        &[
            "path", "audioPath", "soundPath", "filePath", "file", "audioFile", "sound",
        ],
    )
    .unwrap_or_default();
    if path.is_empty() {
        crate::log_warn!("音频播放动作缺少文件路径");
        return;
    }
    let volume = s_i64(settings, &["volume", "Volume"]).unwrap_or(100).clamp(0, 100);
    let fade = s_i64(settings, &["fadeDuration"]).unwrap_or(0).max(0);
    let loop_play = s_bool(settings, &["loop", "Loop"]).unwrap_or(false)
        || s_i64(settings, &["actionType"]).map(|v| v == 1).unwrap_or(false);

    if dry_run {
        crate::log_info!(
            "[试运行] 播放音频：{} 音量={} 淡入={}ms 循环={}",
            path, volume, fade, loop_play
        );
        return;
    }

    let seq = ALIAS_SEQ.fetch_add(1, Ordering::Relaxed);
    let alias = format!("n4snd{}", seq);
    // 打开文件（MCI 按扩展名自动选择解码器）
    if mci(&format!("open \"{}\" alias {}", path, alias)) != 0 {
        crate::log_error!("音频打开失败：{}", path);
        return;
    }
    if fade > 0 {
        set_volume(&alias, 0);
    } else {
        set_volume(&alias, volume);
    }
    let play_cmd = if loop_play {
        format!("play {} repeat", alias)
    } else {
        format!("play {}", alias)
    };
    if mci(&play_cmd) != 0 {
        crate::log_error!("音频播放失败：{}", path);
        mci(&format!("close {}", alias));
        return;
    }
    if let Ok(mut list) = aliases().lock() {
        list.push(alias.clone());
    }
    crate::log_info!("播放音频：{}（音量 {}，别名 {}）", path, volume, alias);

    // 淡入在独立线程完成，避免阻塞按键响应
    if fade > 0 {
        let alias_fade = alias.clone();
        std::thread::spawn(move || {
            let steps = 10u64;
            let step_ms = (fade as u64 / steps).max(10);
            for i in 1..=steps {
                std::thread::sleep(Duration::from_millis(step_ms));
                set_volume(&alias_fade, volume * i as i64 / steps as i64);
            }
        });
    }
}

/// 停止所有音频
pub fn stop_all(dry_run: bool) {
    if dry_run {
        crate::log_info!("[试运行] 停止全部音频");
        return;
    }
    let list: Vec<String> = {
        let mut guard = aliases().lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *guard)
    };
    for alias in &list {
        mci(&format!("stop {}", alias));
        mci(&format!("close {}", alias));
    }
    crate::log_info!("已停止 {} 个音频", list.len());
}
