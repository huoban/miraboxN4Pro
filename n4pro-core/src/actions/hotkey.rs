// 热键动作：解析 Settings.Hotkeys[] 并模拟按键

use crate::platform::win::key_press;
use serde_json::Value;

fn flag(entry: &Value, key: &str) -> bool {
    entry.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

pub fn run(settings: &Value, dry_run: bool) {
    let Some(list) = settings.get("Hotkeys").and_then(|v| v.as_array()) else {
        crate::log_warn!("热键动作缺少 Hotkeys 配置");
        return;
    };
    for entry in list {
        let vk = entry.get("VKeyCode").and_then(|v| v.as_i64()).unwrap_or(-1);
        if vk <= 0 || vk > 0xFF {
            continue;
        }
        let ctrl = flag(entry, "KeyCtrl");
        let shift = flag(entry, "KeyShift");
        let alt = flag(entry, "KeyAlt");
        let win = flag(entry, "KeyCmd");
        if dry_run {
            crate::log_info!(
                "[试运行] 热键 VK=0x{:02X} Ctrl={} Shift={} Alt={} Win={}",
                vk, ctrl, shift, alt, win
            );
        } else {
            crate::log_info!(
                "执行热键 VK=0x{:02X} Ctrl={} Shift={} Alt={} Win={}",
                vk, ctrl, shift, alt, win
            );
            key_press(vk as u16, ctrl, shift, alt, win);
        }
    }
}
