// 鼠标事件：模拟单击/双击/移动/滚轮/拖拽，支持修饰键与旋钮滚轮
// 设置键（官方实测）：mouseEventIndex、mouseButtonIndex、xCoordinate、yCoordinate、
// xCoordinateDest、yCoordinateDest、amountOfWheelMovement、openModifier
// 旋钮另用：mouseEventIndexKnob、mouseButtonIndexKnob、xCoordinateKnob、yCoordinateKnob、amountOfWheelMovementknob

use crate::actions::{s_i64, Trigger};
use crate::platform::win::{keybd_event, mouse_button, mouse_move_to, mouse_wheel};
use serde_json::Value;
use std::thread::sleep;
use std::time::Duration;

/// 取设置值（多个候选键）
fn get(settings: &Value, keys: &[&str]) -> Option<i64> {
    s_i64(settings, keys)
}

/// 按下修饰键：openModifier 位序为 Win/Shift/Ctrl/Alt
fn press_modifiers(mods: i64) {
    unsafe {
        if mods & 1 != 0 {
            keybd_event(0x5B, 0, 0, 0); // Win
        }
        if mods & 2 != 0 {
            keybd_event(0x10, 0, 0, 0); // Shift
        }
        if mods & 4 != 0 {
            keybd_event(0x11, 0, 0, 0); // Ctrl
        }
        if mods & 8 != 0 {
            keybd_event(0x12, 0, 0, 0); // Alt
        }
    }
}

/// 释放修饰键（与按下顺序相反）
fn release_modifiers(mods: i64) {
    unsafe {
        if mods & 8 != 0 {
            keybd_event(0x12, 0, 2, 0);
        }
        if mods & 4 != 0 {
            keybd_event(0x11, 0, 2, 0);
        }
        if mods & 2 != 0 {
            keybd_event(0x10, 0, 2, 0);
        }
        if mods & 1 != 0 {
            keybd_event(0x5B, 0, 2, 0);
        }
    }
}

/// 官方按键索引 → 鼠标按键索引：
/// 0=左 1=右 2=竖向滚轮 3=横向滚轮 4=侧键1 5=侧键2
fn click_button(official: i32) -> i32 {
    match official {
        1 => 1,
        4 => 3,
        5 => 4,
        _ => 0,
    }
}

/// 执行一次鼠标事件
pub fn run(settings: &Value, dry_run: bool, trigger: Trigger) {
    let rotate = matches!(trigger, Trigger::Rotate(_));
    let dir = match trigger {
        Trigger::Rotate(d) => d,
        _ => 0,
    };

    // 旋钮旋转优先使用 Knob 专属字段，缺失时回退到基础字段
    let idx_keys: &[&str] = if rotate {
        &["mouseEventIndexKnob", "mouseEventIndex"]
    } else {
        &["mouseEventIndex"]
    };
    let btn_keys: &[&str] = if rotate {
        &["mouseButtonIndexKnob", "mouseButtonIndex"]
    } else {
        &["mouseButtonIndex"]
    };
    let x_keys: &[&str] = if rotate {
        &["xCoordinateKnob", "xCoordinate"]
    } else {
        &["xCoordinate"]
    };
    let y_keys: &[&str] = if rotate {
        &["yCoordinateKnob", "yCoordinate"]
    } else {
        &["yCoordinate"]
    };
    let amount_keys: &[&str] = if rotate {
        &["amountOfWheelMovementknob", "amountOfWheelMovement"]
    } else {
        &["amountOfWheelMovement"]
    };

    // 旋钮旋转默认按竖向滚轮处理
    let event = get(settings, idx_keys).unwrap_or(if rotate { 3 } else { 0 }) as i32;
    let button = get(settings, btn_keys).unwrap_or(0) as i32;
    let x = get(settings, x_keys).unwrap_or(0) as i32;
    let y = get(settings, y_keys).unwrap_or(0) as i32;
    let x_dest = get(settings, &["xCoordinateDest"]).unwrap_or(0) as i32;
    let y_dest = get(settings, &["yCoordinateDest"]).unwrap_or(0) as i32;
    let mut amount = get(settings, amount_keys).unwrap_or(120) as i32;
    if rotate && dir < 0 {
        amount = -amount;
    }
    let modifiers = get(settings, &["openModifier"]).unwrap_or(0);

    if dry_run {
        crate::log_info!(
            "[试运行] 鼠标事件 类型={} 按键={} 坐标=({},{}) 目标=({},{}) 滚轮量={} 修饰={}",
            event, button, x, y, x_dest, y_dest, amount, modifiers
        );
        return;
    }

    press_modifiers(modifiers);
    let mb = click_button(button);
    match event {
        0 => {
            // 单击
            mouse_move_to(x, y);
            sleep(Duration::from_millis(15));
            mouse_button(mb, true);
            sleep(Duration::from_millis(20));
            mouse_button(mb, false);
        }
        1 => {
            // 双击
            mouse_move_to(x, y);
            sleep(Duration::from_millis(15));
            for _ in 0..2 {
                mouse_button(mb, true);
                sleep(Duration::from_millis(20));
                mouse_button(mb, false);
                sleep(Duration::from_millis(60));
            }
        }
        2 => {
            // 移动
            mouse_move_to(x, y);
        }
        3 => {
            // 滚轮：按键 3 表示横向滚轮，其余按竖向处理
            mouse_wheel(amount, button == 3);
        }
        4 => {
            // 拖拽：起点按下 → 移动 → 终点释放
            mouse_move_to(x, y);
            sleep(Duration::from_millis(15));
            mouse_button(mb, true);
            sleep(Duration::from_millis(20));
            mouse_move_to(x_dest, y_dest);
            sleep(Duration::from_millis(30));
            mouse_button(mb, false);
        }
        _ => crate::log_warn!("未知鼠标事件类型 {}", event),
    }
    release_modifiers(modifiers);
    crate::log_info!(
        "鼠标事件完成：类型={} 按键={} 坐标=({},{}) 滚轮量={}",
        event, button, x, y, amount
    );
}
