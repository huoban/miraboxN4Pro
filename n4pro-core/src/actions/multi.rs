// 多动作：串行/并行执行子动作，支持延时

use crate::actions::{execute, s_i64, Effect};
use crate::config::manifest::parse_action;
use serde_json::Value;

fn find_children(settings: &Value) -> Option<Vec<Value>> {
    for key in ["Actions", "actions", "list", "children", "Routine", "routine"] {
        if let Some(arr) = settings.get(key).and_then(|v| v.as_array()) {
            return Some(arr.clone());
        }
    }
    None
}

pub fn run(settings: &Value, dry_run: bool, depth: u8) -> Effect {
    if depth > 8 {
        crate::log_warn!("多动作嵌套过深，已停止");
        return Effect::None;
    }
    if let Some(children) = find_children(settings) {
        let parallel = settings
            .get("parallel")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let mut effect = Effect::None;
        if parallel {
            let mut handles = Vec::new();
            for child in children {
                let def = parse_action(&child);
                handles.push(std::thread::spawn(move || execute(&def, dry_run, depth + 1)));
            }
            for h in handles {
                if let Ok(e) = h.join() {
                    if effect == Effect::None && e != Effect::None {
                        effect = e;
                    }
                }
            }
        } else {
            for child in children {
                let def = parse_action(&child);
                let e = execute(&def, dry_run, depth + 1);
                if e != Effect::None {
                    effect = e;
                }
            }
        }
        return effect;
    }

    if let Some(ms) = s_i64(settings, &["delay", "time", "ms"]) {
        if dry_run {
            crate::log_info!("[试运行] 多动作延时 {} ms", ms);
        } else {
            crate::log_info!("多动作延时 {} ms", ms);
            std::thread::sleep(std::time::Duration::from_millis(ms.max(0) as u64));
        }
    }
    Effect::None
}
