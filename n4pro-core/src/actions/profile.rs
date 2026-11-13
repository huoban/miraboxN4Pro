// 场景动作：场景切换（旋钮轮换）、返回上级、进入子场景

use crate::actions::{s_str, Effect};
use serde_json::Value;

/// 执行场景动作（旋转轮换由 app 处理）
pub fn run(uuid: &str, settings: &Value) -> Effect {
    if uuid.contains("profile.backtoparent") {
        return Effect::BackToParent;
    }
    if uuid.contains("profile.openchild") {
        let target = s_str(
            settings,
            &["ProfileUUID", "profileUUID", "DeviceUUID", "uuid", "path"],
        )
        .unwrap_or_default();
        if target.is_empty() {
            crate::log_warn!("进入子场景动作缺少目标 UUID");
            return Effect::None;
        }
        return Effect::GotoPage(target);
    }
    Effect::None
}
