// 工具箱内置动作：Super Hotkeys / Hotkey Switch / Text / Password / Close
// 官方设置键：Hotkeys、executionTimes、isCmd、appArguments、openShowState、
// pastedText、isSendingEnter、useCopyAndPaste、forceOut

use crate::actions::{hotkey, s_bool, s_i64, s_str};
use crate::config::ActionDef;
use crate::platform::win::{close_app, clipboard_set_text, key_press, run_hidden};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Hotkey Switch 的当前状态（按动作实例 ID 持久化）
static SWITCH_STATE: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();

fn switch_map() -> &'static Mutex<HashMap<String, usize>> {
    SWITCH_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 执行一组热键（list 为 Hotkeys 数组），统一走扫描码 + 左右修饰键
fn run_hotkey_list(list: &[Value], dry_run: bool) {
    for entry in list {
        hotkey::send_entry(entry, dry_run);
    }
}

/// Super Hotkeys：扫描码方式模拟物理键盘，支持重复次数与命令模式
pub fn super_hotkey(settings: &Value, dry_run: bool) {
    let is_cmd = s_bool(settings, &["isCmd"]).unwrap_or(false);
    let path = s_str(settings, &["path", "command", "cmd", "appPath"]).unwrap_or_default();
    if is_cmd && !path.is_empty() {
        let args = s_str(settings, &["appArguments", "arguments"]).unwrap_or_default();
        let line = if args.is_empty() { path } else { format!("{} {}", path, args) };
        if dry_run {
            crate::log_info!("[试运行] Super Hotkeys 命令：{}", line);
        } else {
            crate::log_info!("Super Hotkeys 命令：{}", line);
            run_hidden(&line);
        }
        return;
    }
    let Some(list) = settings.get("Hotkeys").and_then(|v| v.as_array()) else {
        crate::log_warn!("Super Hotkeys 缺少 Hotkeys 配置");
        return;
    };
    let times = s_i64(settings, &["executionTimes", "times"]).unwrap_or(1).clamp(1, 20);
    for _ in 0..times {
        run_hotkey_list(list, dry_run);
    }
}

/// Hotkey Switch：执行当前组热键后切换到下一组
pub fn hotkey_switch(def: &ActionDef, dry_run: bool) {
    // 兼容两种结构：Hotkeys 为二维数组（多组）或 Hotkeys/Hotkeys2 两个字段
    let mut groups: Vec<Vec<Value>> = Vec::new();
    if let Some(arr) = def.settings.get("Hotkeys").and_then(|v| v.as_array()) {
        if arr.iter().all(|v| v.is_array()) {
            groups = arr
                .iter()
                .map(|v| v.as_array().cloned().unwrap_or_default())
                .collect();
        } else {
            groups.push(arr.clone());
        }
    }
    if let Some(arr) = def.settings.get("Hotkeys2").and_then(|v| v.as_array()) {
        groups.push(arr.clone());
    }
    if groups.is_empty() {
        crate::log_warn!("Hotkey Switch 缺少 Hotkeys 配置");
        return;
    }

    let key = if def.action_id.is_empty() {
        format!("{}#{}", def.uuid, def.name)
    } else {
        def.action_id.clone()
    };
    let idx = {
        let mut map = switch_map().lock().unwrap_or_else(|e| e.into_inner());
        let cur = *map.get(&key).unwrap_or(&0);
        map.insert(key, (cur + 1) % groups.len());
        cur % groups.len()
    };
    crate::log_info!("Hotkey Switch 执行第 {}/{} 组", idx + 1, groups.len());
    run_hotkey_list(&groups[idx], dry_run);
}

/// Text / Password：写入剪贴板并粘贴，可选回车
pub fn paste_text(settings: &Value, dry_run: bool, password: bool) {
    let text = s_str(
        settings,
        &["pastedText", "text", "Text", "password", "Password", "content"],
    )
    .unwrap_or_default();
    if text.is_empty() {
        crate::log_warn!("{}动作缺少文本内容", if password { "密码" } else { "文本" });
        return;
    }
    let enter = s_bool(settings, &["isSendingEnter", "sendEnter"]).unwrap_or(false);
    if dry_run {
        crate::log_info!(
            "[试运行] {}输出 {} 个字符，回车={}",
            if password { "密码" } else { "文本" },
            text.chars().count(),
            enter
        );
        return;
    }
    if !clipboard_set_text(&text) {
        crate::log_error!("剪贴板写入失败");
        return;
    }
    std::thread::sleep(std::time::Duration::from_millis(30));
    key_press(0x56, true, false, false, false); // Ctrl+V
    if enter {
        std::thread::sleep(std::time::Duration::from_millis(40));
        key_press(0x0D, false, false, false, false);
    }
    crate::log_info!(
        "{}已输出（{} 个字符）",
        if password { "密码" } else { "文本" },
        text.chars().count()
    );
}

/// Close：按路径关闭对应进程
pub fn close(settings: &Value, dry_run: bool) {
    let path = s_str(settings, &["path", "appPath", "process", "name"]).unwrap_or_default();
    if path.is_empty() {
        crate::log_warn!("关闭动作缺少 path");
        return;
    }
    let force = s_bool(settings, &["forceOut", "force"]).unwrap_or(false);
    if dry_run {
        crate::log_info!("[试运行] 关闭：{} 强制={}", path, force);
        return;
    }
    crate::log_info!("关闭：{} 强制={}", path, force);
    if !close_app(&path, force) {
        crate::log_warn!("关闭命令执行失败：{}", path);
    }
}
