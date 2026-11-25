// 热键动作：解析 Settings.Hotkeys[]，支持左右修饰键、Alt(KeyOption) 与多主键组合
// 统一使用扫描码发送，兼容普通窗口与游戏

use crate::actions::s_i64;
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
        // 文档 05 官方字段为 KeyWin/RKeyWin；同时兼容旧实现的 KeyCmd/RKeyCmd 写法
        win: flag(entry, "KeyWin") || flag(entry, "KeyCmd"),
        r_ctrl: flag(entry, "RKeyCtrl"),
        r_shift: flag(entry, "RKeyShift"),
        r_alt: flag(entry, "RKeyOption") || flag(entry, "RKeyAlt"),
        r_win: flag(entry, "RKeyWin") || flag(entry, "RKeyCmd"),
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

/// 归一化热键条目：兼容官方三种历史结构
/// 1) `Hotkeys: [ {...}, ... ]` 多槽位（当前版本固定 3 条，空槽 VKeyCode=-1）
/// 2) `Hotkeys: { ... }` 单条对象
/// 3) `Settings` 本身即 `Key*`/`VKeyCode` 的旧版扁平结构（无 Hotkeys）
pub fn entries(settings: &Value) -> Vec<Value> {
    match settings.get("Hotkeys") {
        Some(Value::Array(list)) => list.clone(),
        Some(obj @ Value::Object(_)) => vec![obj.clone()],
        _ => {
            let flat = ["VKeyCode", "VKeyCodes", "KeyCtrl", "KeyShift", "KeyOption", "KeyCmd"]
                .iter()
                .any(|k| settings.get(*k).is_some());
            if flat {
                vec![settings.clone()]
            } else {
                Vec::new()
            }
        }
    }
}

pub fn run(settings: &Value, dry_run: bool) {
    let list = entries(settings);
    if !list.iter().any(|e| !parse_vks(e).is_empty()) {
        crate::log_warn!("热键动作缺少有效按键配置");
        return;
    }
    // 普通热键同样支持 executionTimes 重复执行次数（与 Super Hotkeys 一致，上限 20）
    let times = s_i64(settings, &["executionTimes", "times"]).unwrap_or(1).clamp(1, 20);
    for _ in 0..times {
        for entry in &list {
            send_entry(entry, dry_run);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 当前版本：Hotkeys 固定 3 槽位，空槽 VKeyCode=-1
    #[test]
    fn 解析多槽位热键() {
        let s = json!({"Coalesce": true, "Hotkeys": [
            {"VKeyCode": 77, "VKeyCodes": [], "KeyCtrl": true, "KeyShift": true},
            {"VKeyCode": -1, "VKeyCodes": []},
            {"VKeyCode": -1, "VKeyCodes": []}
        ]});
        let list = entries(&s);
        assert_eq!(list.len(), 3);
        assert_eq!(parse_vks(&list[0]), vec![77]);
        assert!(parse_vks(&list[1]).is_empty());
    }

    /// 旧版扁平结构：Settings 直接是 Key*（如 WZL914S3 场景）
    #[test]
    fn 解析扁平热键() {
        let s = json!({"KeyCmd": true, "VKeyCode": 109, "KeyCtrl": false});
        let list = entries(&s);
        assert_eq!(list.len(), 1);
        assert_eq!(parse_vks(&list[0]), vec![109]);
    }

    /// 单条对象形态
    #[test]
    fn 解析单条对象热键() {
        let s = json!({"Hotkeys": {"VKeyCode": 32, "KeyShift": true}});
        let list = entries(&s);
        assert_eq!(list.len(), 1);
        assert_eq!(parse_vks(&list[0]), vec![32]);
    }

    /// 多主键组合：VKeyCode 会重复出现在 VKeyCodes 首位，需去重且保持顺序
    #[test]
    fn 多主键组合去重保序() {
        let s = json!({"VKeyCode": 33, "VKeyCodes": [33, 34, 36, 8, 220, 79]});
        assert_eq!(parse_vks(&s), vec![33, 34, 36, 8, 220, 79]);
    }

    /// 左右修饰键与 ⌘(KeyCmd) 映射
    #[test]
    fn 解析左右修饰键() {
        let s = json!({
            "VKeyCode": 39,
            "KeyCtrl": true, "KeyShift": true, "KeyOption": true, "KeyCmd": true,
            "RKeyCtrl": true, "RKeyShift": true, "RKeyOption": true
        });
        let m = parse_mods(&s);
        assert!(m.ctrl && m.shift && m.alt && m.win);
        assert!(m.r_ctrl && m.r_shift && m.r_alt && !m.r_win);
    }
}


