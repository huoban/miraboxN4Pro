// 动作执行器：按 UUID/设置分派到具体实现
// 说明：插件类动作不加载插件进程，记录日志后跳过（按需求不做插件适配）

pub mod hotkey;
pub mod media;
pub mod multi;
pub mod net;
pub mod open;

use crate::config::ActionDef;
use serde_json::Value;

/// 需要主程序处理的页面/场景效果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    NextPage,
    PrevPage,
    SwitchScene(String),
}

/// 读取字符串设置（多个候选键）
pub fn s_str(settings: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(v) = settings.get(*key) {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
            if v.is_number() {
                return Some(v.to_string());
            }
        }
    }
    None
}

pub fn s_i64(settings: &Value, keys: &[&str]) -> Option<i64> {
    for key in keys {
        if let Some(v) = settings.get(*key) {
            if let Some(n) = v.as_i64() {
                return Some(n);
            }
            if let Some(s) = v.as_str() {
                if let Ok(n) = s.trim().parse::<i64>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

pub fn s_bool(settings: &Value, keys: &[&str]) -> Option<bool> {
    for key in keys {
        if let Some(v) = settings.get(*key) {
            if let Some(b) = v.as_bool() {
                return Some(b);
            }
            if let Some(s) = v.as_str() {
                return Some(s == "true" || s == "1");
            }
        }
    }
    None
}

/// 执行一个动作；返回需要主程序处理的效果
pub fn execute(def: &ActionDef, dry_run: bool, depth: u8) -> Effect {
    let uuid = def.uuid.to_lowercase();

    // 插件类动作：直接跳过（不启动 Node 插件）
    if is_plugin_uuid(&uuid) {
        crate::log_info!("跳过插件动作：{}（{}）", def.name, def.uuid);
        return Effect::None;
    }

    if uuid.contains("profile.rotate") {
        let target = s_str(&def.settings, &["ProfileUUID", "profileUUID", "profile"]).unwrap_or_default();
        if target.is_empty() {
            crate::log_warn!("场景切换动作缺少 ProfileUUID，已忽略");
            return Effect::None;
        }
        return Effect::SwitchScene(target);
    }
    if uuid.contains("page.change") {
        let forward = s_bool(&def.settings, &["Next", "forward"]).unwrap_or(true);
        return if forward { Effect::NextPage } else { Effect::PrevPage };
    }
    if uuid.contains("http") {
        net::http(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("osc") {
        net::osc(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("network.udp") || uuid.contains("system.udp") {
        net::udp_or_osc(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("website") {
        open::run(&def.settings, dry_run, true);
        return Effect::None;
    }
    if uuid.contains("system.open") || uuid.ends_with(".open") {
        open::run(&def.settings, dry_run, false);
        return Effect::None;
    }
    if uuid.contains("multimedia") {
        media::run(&def.settings, dry_run, false);
        return Effect::None;
    }
    if uuid.contains("volume") {
        media::run(&def.settings, dry_run, true);
        return Effect::None;
    }
    if uuid.contains("delay") {
        let ms = s_i64(&def.settings, &["delay", "Delay", "time", "ms", "duration"]).unwrap_or(1000);
        if !dry_run {
            std::thread::sleep(std::time::Duration::from_millis(ms.max(0) as u64));
        }
        return Effect::None;
    }
    if uuid.contains("multiactions") {
        return multi::run(&def.settings, dry_run, depth);
    }
    if uuid.contains("hotkey") {
        hotkey::run(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("system.close") {
        crate::log_info!("忽略退出程序动作（重制版不提供退出按钮）");
        return Effect::None;
    }

    crate::log_warn!("未识别的动作，已忽略：{}（{}）", def.name, def.uuid);
    Effect::None
}

/// 插件类 UUID 判定：除内置热点动作外的第三方插件
fn is_plugin_uuid(uuid: &str) -> bool {
    let builtin_prefix = [
        "com.hotspot.streamdock.system.",
        "com.hotspot.streamdock.profile.",
        "com.hotspot.streamdock.page.",
        "com.hotspot.streamdock.multiactions.",
        "com.hotspot.streamdock.network.",
        "com.hotspot.streamdock.touchbar.",
        "com.hotspot.streamdock.quickcontrol.",
        "com.hotspot.streamdock.quicktool.",
        "com.hotspot.streamdock.device.",
        "com.hotspot.streamdock.plain.text",
        "com.hotspot.streamdock.mouse.event",
        "com.hotspot.streamdock.soundboard",
        "com.hotspot.streamdock.browser",
        "com.hotspot.streamdock.multiactions",
    ];
    if builtin_prefix.iter().any(|p| uuid.starts_with(p)) {
        // 文本/鼠标/音板等由插件实现，重制版未适配，按插件处理
        return uuid.contains("plain.text")
            || uuid.contains("mouse.event")
            || uuid.contains("soundboard")
            || uuid.contains("device.brightness")
            || uuid.contains("device.k1pro")
            || uuid.contains("browser");
    }
    uuid.starts_with("com.mirabox.")
        || uuid.starts_with("com.streamdock.")
        || uuid.starts_with("com.hotspot.streamdock.obs")
        || uuid.starts_with("com.hotspot.streamdock.vmix")
        || uuid.starts_with("com.hotspot.streamdock.youtube")
}
