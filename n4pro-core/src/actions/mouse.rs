// 鼠标事件：模拟单击/双击/移动/滚轮/拖拽，支持修饰键与旋钮滚轮
// 设置键（官方实测）：mouseEventIndex、mouseButtonIndex、xCoordinate、yCoordinate、
// xCoordinateDest、yCoordinateDest、amountOfWheelMovement、openModifier
// 旋钮另用：mouseEventIndexKnob、mouseButtonIndexKnob、xCoordinateKnob、yCoordinateKnob、amountOfWheelMovementknob

use crate::actions::{s_i64, Trigger};
use crate::platform::win::{cursor_pos, keybd_event, mouse_button, mouse_move_to, mouse_wheel};
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

/// 旋钮旋转 → 鼠标相对位移
/// 官方模板用 (xCoordinateKnob, yCoordinateKnob) 直接表示每格的位移量：
/// (1,0)=X 轴每格 1px、(0,1)=Y 轴每格 1px，正负表示方向，(1,1)=斜向 1px；
/// 缺失时按 mouseButtonIndex 推断轴（2=竖向/Y、3=横向/X），步长取 1px。
fn rotate_move(settings: &Value, dir: i8) -> (i32, i32) {
    let vx = get(settings, &["xCoordinateKnob", "xCoordinate"]).unwrap_or(0);
    let vy = get(settings, &["yCoordinateKnob", "yCoordinate"]).unwrap_or(0);
    let (vx, vy) = if vx != 0 || vy != 0 {
        (vx, vy)
    } else if get(settings, &["mouseButtonIndexKnob", "mouseButtonIndex"]).unwrap_or(2) == 3 {
        (1, 0)
    } else {
        (0, 1)
    };
    // 顺时针为正方向，逆时针取反；位移量即配置数值
    let sign = if dir < 0 { -1 } else { 1 };
    ((vx * sign) as i32, (vy * sign) as i32)
}

/// 执行一次鼠标事件
pub fn run(settings: &Value, dry_run: bool, trigger: Trigger) {
    // 旋钮旋转：按配置在鼠标当前位置做相对位移（官方旋钮的“移动鼠标”语义，非滚轮）
    if let Trigger::Rotate(dir) = trigger {
        let (dx, dy) = rotate_move(settings, dir);
        if dx == 0 && dy == 0 {
            crate::log_warn!("旋钮移动鼠标：位移为 0，检查 xCoordinateKnob/yCoordinateKnob");
            return;
        }
        let (cx, cy) = cursor_pos();
        let (nx, ny) = (cx + dx, cy + dy);
        if dry_run {
            crate::log_info!(
                "[试运行] 旋钮移动鼠标 位移=({},{}) 位置=({},{})",
                dx, dy, nx, ny
            );
            return;
        }
        mouse_move_to(nx, ny);
        crate::log_info!(
            "鼠标事件完成：旋钮移动鼠标 位移=({},{}) 位置=({},{})",
            dx, dy, nx, ny
        );
        return;
    }

    let event = get(settings, &["mouseEventIndex"]).unwrap_or(0) as i32;
    let button = get(settings, &["mouseButtonIndex"]).unwrap_or(0) as i32;
    // 未配置坐标时保持光标不动（旋钮按下等场景只有 Knob 字段，不应跳到屏幕左上角）
    let has_pos = settings.get("xCoordinate").is_some() || settings.get("yCoordinate").is_some();
    let x = get(settings, &["xCoordinate"]).unwrap_or(0) as i32;
    let y = get(settings, &["yCoordinate"]).unwrap_or(0) as i32;
    let x_dest = get(settings, &["xCoordinateDest"]).unwrap_or(0) as i32;
    let y_dest = get(settings, &["yCoordinateDest"]).unwrap_or(0) as i32;
    let amount = get(settings, &["amountOfWheelMovement"]).unwrap_or(120) as i32;
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
            if has_pos {
                mouse_move_to(x, y);
                sleep(Duration::from_millis(15));
            }
            mouse_button(mb, true);
            sleep(Duration::from_millis(20));
            mouse_button(mb, false);
        }
        1 => {
            // 双击
            if has_pos {
                mouse_move_to(x, y);
                sleep(Duration::from_millis(15));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn 旋钮坐标向量映射位移轴与方向() {
        // 模板 1,0（X轴）：位移量等于配置数值 1px
        let x_axis = json!({"xCoordinateKnob": 1, "yCoordinateKnob": 0, "mouseButtonIndex": 3});
        assert_eq!(rotate_move(&x_axis, 1), (1, 0));
        assert_eq!(rotate_move(&x_axis, -1), (-1, 0));
        // 模板 2,0（Y轴）
        let y_axis = json!({"xCoordinateKnob": 0, "yCoordinateKnob": 1, "mouseButtonIndex": 2});
        assert_eq!(rotate_move(&y_axis, 1), (0, 1));
        assert_eq!(rotate_move(&y_axis, -1), (0, -1));
        // 配置数值即位移量（放大步长只需改配置）
        let big = json!({"xCoordinateKnob": 5, "yCoordinateKnob": 0});
        assert_eq!(rotate_move(&big, 1), (5, 0));
        // 负向量翻转方向
        let neg = json!({"xCoordinateKnob": -1, "yCoordinateKnob": 0});
        assert_eq!(rotate_move(&neg, 1), (-1, 0));
        // 同时配置 X/Y 时为斜向
        let both = json!({"xCoordinateKnob": 1, "yCoordinateKnob": 1});
        assert_eq!(rotate_move(&both, 1), (1, 1));
        // 缺省字段：竖向/Y、1px
        assert_eq!(rotate_move(&json!({}), 1), (0, 1));
    }
}
