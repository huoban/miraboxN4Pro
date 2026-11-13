// 打开动作：程序、文件、文件夹、网址

use crate::actions::s_str;
use crate::platform::win::shell_open;
use serde_json::Value;

pub fn run(settings: &Value, dry_run: bool, website: bool) {
    let path = s_str(settings, &["path", "url", "Path", "URL"]).unwrap_or_default();
    if path.is_empty() {
        crate::log_warn!("打开动作缺少 path/url");
        return;
    }
    let target = if website {
        let browser = s_str(settings, &["browser"]).unwrap_or_default();
        if !browser.is_empty() && browser != "default" {
            format!("{} {}", browser, path)
        } else {
            path
        }
    } else {
        path
    };
    if dry_run {
        crate::log_info!("[试运行] 打开：{}", target);
        return;
    }
    crate::log_info!("打开：{}", target);
    if !shell_open(&target) {
        crate::log_error!("打开失败：{}", target);
    }
}
