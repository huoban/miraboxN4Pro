// 页面动作：上一页/下一页/跳转/页码指示/旋钮翻页

use crate::actions::{s_str, Effect, Trigger};
use serde_json::Value;

/// 执行页面动作；旋钮旋转按方向翻页
pub fn run(uuid: &str, settings: &Value, trigger: Trigger) -> Effect {
    if uuid.contains("page.next") {
        return Effect::NextPage;
    }
    if uuid.contains("page.previous") {
        return Effect::PrevPage;
    }
    if uuid.contains("page.change") {
        // 旋钮：顺时针下一页，逆时针上一页
        if let Trigger::Rotate(dir) = trigger {
            return if dir >= 0 { Effect::NextPage } else { Effect::PrevPage };
        }
        return Effect::NextPage;
    }
    if uuid.contains("page.goto") {
        let target = s_str(
            settings,
            &["PageIndex", "pageIndex", "page", "PageUUID", "index"],
        )
        .unwrap_or_default();
        if target.is_empty() {
            crate::log_warn!("跳转页面动作缺少目标页");
            return Effect::None;
        }
        return Effect::GotoPage(target);
    }
    if uuid.contains("page.indicator") {
        // 页码指示仅影响图标显示，点击无动作
        return Effect::None;
    }
    Effect::None
}
