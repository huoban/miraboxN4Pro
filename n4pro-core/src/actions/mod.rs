// 动作执行器：按 UUID/设置分派到具体实现
// 说明：插件类动作不加载插件进程，记录日志后跳过；本版本已内置工具箱、操作流、
// 音频播放器、切换音频设备、触控条（Windows 可用项）与鼠标事件。

pub mod audio_device;
pub mod hotkey;
pub mod media;
pub mod mouse;
pub mod multi;
pub mod net;
pub mod open;
pub mod pages;
pub mod profile;
pub mod soundboard;
pub mod toolbox;
pub mod touchbar;

use crate::config::ActionDef;
use serde_json::Value;

/// 触发方式：按键按下 / 旋钮旋转（方向）/ 长按
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Press,
    Rotate(i8),
    LongPress,
}

/// 需要主程序处理的页面/场景/设备效果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    NextPage,
    PrevPage,
    /// 跳转页面：值为页码或页面 UUID
    GotoPage(String),
    /// 返回场景根页
    BackToParent,
    SwitchScene(String),
    /// 调整亮度（增量，正值调亮）
    BrightnessDelta(i32),
    /// 切换设备息屏
    DeviceSleepToggle,
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
            if let Some(n) = v.as_i64() {
                return Some(n != 0);
            }
        }
    }
    None
}

/// 执行一个动作；返回需要主程序处理的效果
pub fn execute(def: &ActionDef, dry_run: bool, depth: u8, trigger: Trigger) -> Effect {
    let uuid = def.uuid.to_lowercase();

    // 切换音频设备（原生插件，单独处理）
    if uuid.starts_with("com.mirabox.streamdock.switchaudio") {
        audio_device::switch(&def.settings, dry_run);
        return Effect::None;
    }

    // 插件类动作：直接跳过（不启动 Node 插件）
    if is_plugin_uuid(&uuid) {
        crate::log_info!("跳过插件动作：{}（{}）", def.name, def.uuid);
        return Effect::None;
    }

    // 操作流（多动作）
    if uuid.contains("multiactions.") {
        if uuid.contains("routine") {
            return multi::routine(def, dry_run, depth);
        }
        if uuid.contains("delay") {
            multi::delay(&def.settings, dry_run);
            return Effect::None;
        }
        if uuid.contains("lunbo") {
            return multi::carousel(def, dry_run, depth);
        }
        if uuid.contains("toggle") {
            return multi::toggle(def, dry_run, depth);
        }
        if uuid.contains("actionwheel") || uuid.contains("actiontrigger") {
            return multi::knob_select(def, dry_run, depth, trigger);
        }
    }
    if uuid.contains("knoboperatinggroup") {
        return multi::knob_select(def, dry_run, depth, trigger);
    }

    // 鼠标事件
    if uuid.contains("mouse.event") {
        mouse::run(&def.settings, dry_run, trigger);
        return Effect::None;
    }

    // 音频播放器
    if uuid.contains("soundboard.playaudio") {
        soundboard::play(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("soundboard.stopaudioplay") {
        soundboard::stop_all(dry_run);
        return Effect::None;
    }

    // 触控条（Windows 可用项）
    if uuid.contains("touchbar.") {
        touchbar::run(&uuid, dry_run, trigger);
        return Effect::None;
    }

    // 页面与场景
    if uuid.contains("page.") {
        return pages::run(&uuid, &def.settings, trigger);
    }
    if uuid.contains("profile.rotate") {
        // 场景轮换由 app 处理（按键与旋钮），多动作内忽略
        return Effect::None;
    }
    if uuid.contains("profile.") {
        return profile::run(&uuid, &def.settings);
    }

    // 设备亮度/息屏
    if uuid.contains("device.brightness") {
        let delta = match trigger {
            Trigger::Rotate(d) => d as i32 * 10,
            _ => match s_i64(&def.settings, &["actionIdx"]).unwrap_or(0) {
                1 => -10,
                _ => 10,
            },
        };
        return Effect::BrightnessDelta(delta);
    }
    if uuid.contains("device.devsleep") {
        return Effect::DeviceSleepToggle;
    }

    // 工具箱
    if uuid.contains("super.hotkey") {
        toolbox::super_hotkey(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("hotkeyswitch") {
        toolbox::hotkey_switch(def, dry_run);
        return Effect::None;
    }
    if uuid.contains("system.hotkey") || uuid.ends_with(".hotkey") {
        hotkey::run(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("system.multimedia") {
        media::run(&def.settings, dry_run, false, trigger);
        return Effect::None;
    }
    if uuid.contains("system.volume") {
        media::run(&def.settings, dry_run, true, trigger);
        return Effect::None;
    }
    if uuid.contains("system.openapps") {
        open::run_apps(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("system.open") {
        open::run(&def.settings, dry_run, false);
        return Effect::None;
    }
    if uuid.contains("system.close") {
        toolbox::close(&def.settings, dry_run);
        return Effect::None;
    }
    if uuid.contains("system.text") {
        toolbox::paste_text(&def.settings, dry_run, false);
        return Effect::None;
    }
    if uuid.contains("system.password") {
        toolbox::paste_text(&def.settings, dry_run, true);
        return Effect::None;
    }
    if uuid.contains("system.website") || uuid.contains("website") {
        open::run(&def.settings, dry_run, true);
        return Effect::None;
    }

    // 网络动作
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

    crate::log_warn!("未识别的动作，已忽略：{}（{}）", def.name, def.uuid);
    Effect::None
}

/// 插件类 UUID 判定：除已内置实现的动作外的第三方插件
fn is_plugin_uuid(uuid: &str) -> bool {
    // 已内置实现，不按插件跳过
    if uuid.contains("mouse.event")
        || uuid.contains("soundboard")
        || uuid.contains("device.brightness")
        || uuid.contains("device.devsleep")
        || uuid.starts_with("com.mirabox.streamdock.switchaudio")
    {
        return false;
    }
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
        // 文本显示/音板/亮度之外的插件实现，重制版未适配
        return uuid.contains("plain.text")
            || uuid.contains("device.k1pro")
            || uuid.contains("browser");
    }
    uuid.starts_with("com.mirabox.")
        || uuid.starts_with("com.streamdock.")
        || uuid.starts_with("com.hotspot.streamdock.obs")
        || uuid.starts_with("com.hotspot.streamdock.vmix")
        || uuid.starts_with("com.hotspot.streamdock.youtube")
}
