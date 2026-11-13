// 触控条动作（Windows 可用项）：系统快捷键、媒体键、睡眠
// 说明：Siri/Launchpad/调度中心/聚焦搜索为 macOS 专属，Windows 下跳过

use crate::actions::Trigger;
use crate::platform::win::{key_press, media_key, suspend_pc};

const VK_MEDIA_NEXT: u16 = 0xB0;
const VK_MEDIA_PREV: u16 = 0xB1;
const VK_MEDIA_PLAY_PAUSE: u16 = 0xB3;
const VK_VOLUME_MUTE: u16 = 0xAD;
const VK_VOLUME_DOWN: u16 = 0xAE;
const VK_VOLUME_UP: u16 = 0xAF;

/// 执行触控条动作
pub fn run(uuid: &str, dry_run: bool, trigger: Trigger) {
    // 媒体/音量旋钮
    if uuid.contains("touchbar.volume") {
        if let Trigger::Rotate(dir) = trigger {
            let vk = if dir >= 0 { VK_VOLUME_UP } else { VK_VOLUME_DOWN };
            if dry_run {
                crate::log_info!("[试运行] 触控条音量旋钮：{}", if dir >= 0 { "增大" } else { "减小" });
            } else {
                media_key(vk);
            }
        } else if !dry_run {
            media_key(VK_VOLUME_MUTE);
        }
        return;
    }
    if uuid.contains("screen.brightness") || uuid.contains("screenbrightness") {
        crate::log_warn!("触控条屏幕亮度动作在 Windows 下暂不支持：{}", uuid);
        return;
    }

    // 媒体键
    let media = if uuid.contains("previoustrack") {
        Some(VK_MEDIA_PREV)
    } else if uuid.contains("nexttrack") {
        Some(VK_MEDIA_NEXT)
    } else if uuid.contains("playpause") {
        Some(VK_MEDIA_PLAY_PAUSE)
    } else if uuid.contains("fastforward") {
        Some(VK_MEDIA_NEXT)
    } else if uuid.contains("fastrewind") {
        Some(VK_MEDIA_PREV)
    } else if uuid.ends_with("volumeup") {
        Some(VK_VOLUME_UP)
    } else if uuid.ends_with("volumedown") {
        Some(VK_VOLUME_DOWN)
    } else if uuid.ends_with("mute") {
        Some(VK_VOLUME_MUTE)
    } else {
        None
    };
    if let Some(vk) = media {
        if dry_run {
            crate::log_info!("[试运行] 触控条媒体键 VK=0x{:02X}", vk);
        } else {
            media_key(vk);
        }
        return;
    }

    // 系统快捷键
    let hotkey: Option<(u16, bool, bool, bool, bool)> = if uuid.contains("notificationcenter")
        || uuid.contains("dndmode")
    {
        Some((0x4E, false, false, false, true)) // Win+N
    } else if uuid.contains("screenshot") {
        Some((0x53, false, true, false, true)) // Win+Shift+S
    } else if uuid.contains("inputmethod") {
        Some((0x20, false, false, false, true)) // Win+Space
    } else if uuid.contains("showdesktop") || uuid.contains("desktopsaver") {
        Some((0x44, false, false, false, true)) // Win+D
    } else if uuid.contains("screenlock") {
        Some((0x4C, false, false, false, true)) // Win+L
    } else if uuid.contains("dictation") || uuid.contains("speechrecognition") {
        Some((0x48, false, false, false, true)) // Win+H
    } else {
        None
    };
    if let Some((vk, ctrl, shift, alt, win)) = hotkey {
        if dry_run {
            crate::log_info!("[试运行] 触控条快捷键 VK=0x{:02X}", vk);
        } else {
            key_press(vk, ctrl, shift, alt, win);
        }
        return;
    }

    if uuid.contains("touchbar.sleep") {
        if dry_run {
            crate::log_info!("[试运行] 触控条睡眠");
        } else {
            suspend_pc();
        }
        return;
    }

    crate::log_warn!("触控条动作在 Windows 下跳过：{}", uuid);
}
