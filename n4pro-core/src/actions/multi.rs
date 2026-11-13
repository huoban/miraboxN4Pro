// 操作流（多动作）：顺序执行、延时、循环、轮播、切换、旋钮分向
// 官方语义：每个子动作执行前等待其 Settings.delay 毫秒（实测 800/1000/230000）
// 轮播/切换/旋钮选择的索引按动作实例持久化

use crate::actions::{execute, s_bool, s_i64, Effect, Trigger};
use crate::config::ActionDef;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

/// 多动作运行时状态（ActionID → 索引）
static MULTI_STATE: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();

fn state_map() -> &'static Mutex<HashMap<String, usize>> {
    MULTI_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 动作实例键：优先 ActionID，缺失时用 UUID#名称
fn state_key(def: &ActionDef) -> String {
    if def.action_id.is_empty() {
        format!("{}#{}", def.uuid, def.name)
    } else {
        def.action_id.clone()
    }
}

fn read_index(key: &str) -> usize {
    let map = state_map().lock().unwrap_or_else(|e| e.into_inner());
    *map.get(key).unwrap_or(&0)
}

fn write_index(key: &str, value: usize) {
    let mut map = state_map().lock().unwrap_or_else(|e| e.into_inner());
    map.insert(key.to_string(), value);
}

/// 等待子动作自身的延时（毫秒）
fn wait_child_delay(def: &ActionDef, dry_run: bool) {
    let ms = s_i64(&def.settings, &["delay", "Delay"]).unwrap_or(0).max(0);
    if ms == 0 {
        return;
    }
    if dry_run {
        crate::log_info!("[试运行] 子动作延时 {} ms", ms);
    } else {
        std::thread::sleep(Duration::from_millis(ms as u64));
    }
}

/// 依次执行子动作
fn run_children(children: &[ActionDef], dry_run: bool, depth: u8) -> Effect {
    let mut effect = Effect::None;
    for child in children {
        wait_child_delay(child, dry_run);
        let e = execute(child, dry_run, depth + 1, Trigger::Press);
        if e != Effect::None {
            effect = e;
        }
    }
    effect
}

/// Multi Action（顺序执行 + 循环）
pub fn routine(def: &ActionDef, dry_run: bool, depth: u8) -> Effect {
    if depth > 8 {
        crate::log_warn!("多动作嵌套过深，已停止");
        return Effect::None;
    }
    if def.children.is_empty() {
        crate::log_warn!("多动作没有子动作：{}", def.name);
        return Effect::None;
    }

    let cycle = s_bool(&def.settings, &["enablingCycle"]).unwrap_or(false);
    let number = s_i64(&def.settings, &["cycleNumber"]).unwrap_or(1).clamp(1, 1000) as usize;
    let mode = s_i64(&def.settings, &["cycleMode"]).unwrap_or(0);
    let cycle_delay = s_i64(&def.settings, &["cycleDelay"]).unwrap_or(0).max(0);
    // cycleMode=1 视为无限循环（加安全上限，避免场景死循环）
    let rounds = if cycle {
        if mode == 1 {
            1000
        } else {
            number
        }
    } else {
        1
    };

    crate::log_info!(
        "多动作执行：{} 个子动作，循环={} 轮数={} 间隔={}ms",
        def.children.len(),
        cycle,
        rounds,
        cycle_delay
    );

    let mut effect = Effect::None;
    for round in 0..rounds {
        if round > 0 && cycle_delay > 0 {
            if dry_run {
                crate::log_info!("[试运行] 循环间隔 {} ms", cycle_delay);
            } else {
                std::thread::sleep(Duration::from_millis(cycle_delay as u64));
            }
        }
        let e = run_children(&def.children, dry_run, depth);
        if e != Effect::None {
            effect = e;
        }
    }
    effect
}

/// Delay 动作
pub fn delay(settings: &serde_json::Value, dry_run: bool) {
    let ms = s_i64(settings, &["delay", "Delay", "time", "ms"])
        .unwrap_or(1000)
        .max(0);
    if dry_run {
        crate::log_info!("[试运行] 延时 {} ms", ms);
    } else {
        crate::log_info!("延时 {} ms", ms);
        std::thread::sleep(Duration::from_millis(ms as u64));
    }
}

/// Action Carousel（轮播）：每次触发执行下一个子动作
pub fn carousel(def: &ActionDef, dry_run: bool, depth: u8) -> Effect {
    if def.children.is_empty() {
        return Effect::None;
    }
    let key = state_key(def);
    let idx = read_index(&key) % def.children.len();
    write_index(&key, (idx + 1) % def.children.len());
    crate::log_info!("轮播执行第 {}/{} 个动作", idx + 1, def.children.len());
    run_children(&def.children[idx..idx + 1], dry_run, depth)
}

/// Action Cycle（切换）：在两套子动作之间交替
pub fn toggle(def: &ActionDef, dry_run: bool, depth: u8) -> Effect {
    let n = def.children.len();
    if n == 0 {
        return Effect::None;
    }
    let key = state_key(def);
    let state = read_index(&key) % 2;
    write_index(&key, (state + 1) % 2);
    if n == 1 {
        return run_children(&def.children, dry_run, depth);
    }
    let half = (n + 1) / 2;
    let slice = if state == 0 {
        &def.children[..half]
    } else {
        &def.children[half..]
    };
    crate::log_info!("切换执行第 {} 组（{} 个动作）", state + 1, slice.len());
    run_children(slice, dry_run, depth)
}

/// ActionWheel / ActionTrigger / Knob Action Group：
/// 旋转改变选中项，按下执行选中项
pub fn knob_select(def: &ActionDef, dry_run: bool, depth: u8, trigger: Trigger) -> Effect {
    let n = def.children.len();
    if n == 0 {
        return Effect::None;
    }
    let key = state_key(def);
    let mut idx = read_index(&key) % n;
    match trigger {
        Trigger::Rotate(dir) => {
            if dir >= 0 {
                idx = (idx + 1) % n;
            } else {
                idx = (idx + n - 1) % n;
            }
            write_index(&key, idx);
            crate::log_info!("旋钮选择第 {}/{} 个动作", idx + 1, n);
            Effect::None
        }
        _ => {
            crate::log_info!("旋钮执行第 {}/{} 个动作", idx + 1, n);
            run_children(&def.children[idx..idx + 1], dry_run, depth)
        }
    }
}
