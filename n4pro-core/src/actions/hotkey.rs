// 热键动作：解析 Settings.Hotkeys[]，支持左右修饰键、Alt(KeyOption) 与多主键组合
// 统一使用扫描码发送，兼容普通窗口与游戏

use crate::platform::win::{send_hotkey, Mods};
use serde_json::Value;

fn flag(entry: &Value, key: &str) -> bool {
    entry.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

/// 解析修饰键：Key* 为左，RKey* 为右；KeyOption 表示 Alt
fn parse_mods(entry: &Value) -> Mods {
    Mods {
        ctrl: flag(entry, "KeyCtrl"),
        shift: flag(entry, "KeyShift"),
        alt: flag(entry, "KeyOption") || flag(entry, "KeyAlt"),
        win: flag(entry, "KeyCmd"),
        r_ctrl: flag(entry, "RKeyCtrl"),
        r_shift: flag(entry, "RKeyShift"),
        r_alt: flag(entry, "RKeyOption") || flag(entry, "RKeyAlt"),
        r_win: flag(entry, "RKeyCmd"),
    }
}

/// 解析主键列表：VKeyCode + VKeyCodes（数组或逗号字符串）
pub fn parse_vks(entry: &Value) -> Vec<u16> {
    let mut vks: Vec<u16> = Vec::new();
    let mut push = |n: i64| {
        if (1..=0xFF).contains(&n) {
            let vk = n as u16;
            if !vks.contains(&vk) {
                vks.push(vk);
            }
        }
    };
    if let Some(v) = entry.get("VKeyCode").and_then(|v| v.as_i64()) {
        push(v);
    }
    match entry.get("VKeyCodes") {
        Some(Value::Array(arr)) => {
            for item in arr {
                if let Some(n) = item.as_i64() {
                    push(n);
                } else if let Some(s) = item.as_str() {
                    if let Ok(n) = s.trim().parse::<i64>() {
                        push(n);
                    }
                }
            }
        }
        Some(Value::String(s)) if !s.is_empty() => {
            for part in s.split(',') {
                if let Ok(n) = part.trim().parse::<i64>() {
                    push(n);
                }
            }
        }
        _ => {}
    }
    vks
}

/// 修饰键描述（日志用）
fn mods_text(m: &Mods) -> String {
    let mut out = Vec::new();
    if m.ctrl {
        out.push("L-Ctrl");
    }
    if m.r_ctrl {
        out.push("R-Ctrl");
    }
    if m.shift {
        out.push("L-Shift");
    }
    if m.r_shift {
        out.push("R-Shift");
    }
    if m.alt {
        out.push("L-Alt");
    }
    if m.r_alt {
        out.push("R-Alt");
    }
    if m.win {
        out.push("L-Win");
    }
    if m.r_win {
        out.push("R-Win");
    }
    if out.is_empty() {
        "无".to_string()
    } else {
        out.join("+")
    }
}

/// 发送单条热键配置；返回是否识别到有效主键
pub fn send_entry(entry: &Value, dry_run: bool) -> bool {
    let vks = parse_vks(entry);
    if vks.is_empty() {
        return false;
    }
    let mods = parse_mods(entry);
    let vk_text = vks
        .iter()
        .map(|v| format!("0x{:02X}", v))
        .collect::<Vec<_>>()
        .join("+");
    if dry_run {
        crate::log_info!("[试运行] 热键 VK={} 修饰：{}", vk_text, mods_text(&mods));
        return true;
    }
    crate::log_info!("执行热键 VK={} 修饰：{}", vk_text, mods_text(&mods));
    send_hotkey(&vks, mods);
    true
}

pub fn run(settings: &Value, dry_run: bool) {
    let Some(list) = settings.get("Hotkeys").and_then(|v| v.as_array()) else {
        crate::log_warn!("热键动作缺少 Hotkeys 配置");
        return;
    };
    for entry in list {
        send_entry(entry, dry_run);
    }
}
