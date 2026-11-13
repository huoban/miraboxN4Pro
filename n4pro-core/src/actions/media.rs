// 多媒体/音量动作：使用系统媒体键；旋钮旋转按方向切换曲目/调整音量

use crate::actions::{s_i64, Trigger};
use crate::platform::win::media_key;
use serde_json::Value;

const VK_MEDIA_NEXT: u16 = 0xB0;
const VK_MEDIA_PREV: u16 = 0xB1;
const VK_MEDIA_STOP: u16 = 0xB2;
const VK_MEDIA_PLAY_PAUSE: u16 = 0xB3;
const VK_VOLUME_MUTE: u16 = 0xAD;
const VK_VOLUME_DOWN: u16 = 0xAE;
const VK_VOLUME_UP: u16 = 0xAF;

/// volume=true 表示音量类动作，否则为多媒体类
pub fn run(settings: &Value, dry_run: bool, volume: bool, trigger: Trigger) {
    // 旋钮旋转：多媒体切上/下一曲，音量加/减
    if let Trigger::Rotate(dir) = trigger {
        let vk = if volume {
            if dir >= 0 { VK_VOLUME_UP } else { VK_VOLUME_DOWN }
        } else if dir >= 0 {
            VK_MEDIA_NEXT
        } else {
            VK_MEDIA_PREV
        };
        if dry_run {
            crate::log_info!(
                "[试运行] 旋钮{}：VK=0x{:02X}",
                if volume { "音量" } else { "多媒体" },
                vk
            );
        } else {
            crate::log_info!(
                "旋钮{}：VK=0x{:02X}",
                if volume { "音量" } else { "多媒体" },
                vk
            );
            media_key(vk);
        }
        return;
    }

    let idx = s_i64(settings, &["actionIdx", "ActionIdx", "index"]).unwrap_or(0);
    let vk = if volume {
        match idx {
            0 => VK_VOLUME_MUTE,
            1 => VK_VOLUME_UP,
            2 => VK_VOLUME_DOWN,
            _ => {
                crate::log_warn!("未知音量动作编号 {}，已忽略", idx);
                return;
            }
        }
    } else {
        match idx {
            // 官方多媒体编号：21 上一曲 / 22 播放暂停 / 23 下一曲 / 24 停止
            // 25 快进 / 26 快退 / 27 静音 / 28 音量+ / 29 音量-
            21 => VK_MEDIA_PREV,
            22 => VK_MEDIA_PLAY_PAUSE,
            23 => VK_MEDIA_NEXT,
            24 => VK_MEDIA_STOP,
            25 => VK_MEDIA_NEXT,
            26 => VK_MEDIA_PREV,
            27 => VK_VOLUME_MUTE,
            28 => VK_VOLUME_UP,
            29 => VK_VOLUME_DOWN,
            // 兼容旧编号
            0 => VK_MEDIA_PLAY_PAUSE,
            1 => VK_MEDIA_NEXT,
            2 => VK_MEDIA_PREV,
            3 => VK_MEDIA_STOP,
            4 => VK_VOLUME_MUTE,
            5 => VK_VOLUME_UP,
            6 => VK_VOLUME_DOWN,
            _ => {
                crate::log_warn!("未知多媒体动作编号 {}，已忽略", idx);
                return;
            }
        }
    };
    if dry_run {
        crate::log_info!("[试运行] 媒体键 VK=0x{:02X}（编号 {}）", vk, idx);
        return;
    }
    crate::log_info!("执行媒体键 VK=0x{:02X}（编号 {}）", vk, idx);
    media_key(vk);
}
