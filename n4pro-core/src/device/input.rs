// 设备输入协议解析（512 字节）
// 按键:   "ACK" 00 00 "OK" 00 00 idx state      idx 1..10
// 触摸:   idx=0 按下/松开；坐标走 "ACK" 00 "ARX" ... x:BE16 y:BE16
// 旋钮:   0xA1.. 旋转；0x37.. 按下/松开（旋钮 1 实测）
// 手势:   0xB1 触摸结束

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    Key { index: u8, down: bool },
    TouchDown,
    TouchMove { x: i32, y: i32 },
    TouchUp,
    GestureEnd,
    DialRotate { id: u8, dir: i8 },
    DialPress { id: u8, down: bool },
    /// 固件切换触摸条模式：true=Control Bar，false=旋钮模式
    ModeChanged { control_bar: bool },
    /// Control Bar 点击：code=0x39 返回，0x40~0x43 快捷位；x 为触点横坐标
    BarTap { code: u8, x: i32 },
    Unknown { index: u8, state: u8 },
}

fn be16(hi: u8, lo: u8) -> i32 {
    (((hi as i32) << 8) | lo as i32) & 0xFFFF
}

pub fn parse(buf: &[u8]) -> Option<InputEvent> {
    // Windows 读回的数据可能带 1 字节 ReportID（0x00），需要先剥离
    let data = if buf.len() >= 4 && buf[0] == 0x00 && &buf[1..4] == b"ACK" {
        &buf[1..]
    } else {
        buf
    };
    if data.len() < 11 {
        return None;
    }
    // 必须以 "ACK" 开头
    if data[0] != b'A' || data[1] != b'C' || data[2] != b'K' {
        return None;
    }
    // 触摸坐标帧：ACK 00 "ARX" ...
    if data.len() >= 14 && data[3] == 0 && data[4] == b'A' && data[5] == b'R' && data[6] == b'X' {
        let x = be16(data[10], data[11]);
        let y = be16(data[12], data[13]);
        return Some(InputEvent::TouchMove { x, y });
    }
    // ACK 00 00 "OK" 00 00 <idx> <state>
    if data[5] == b'O' && data[6] == b'K' {
        let index = data[9];
        let state = data[10];
        return Some(match index {
            0 => {
                if state == 1 {
                    InputEvent::TouchDown
                } else {
                    InputEvent::TouchUp
                }
            }
            0xB1 => InputEvent::ModeChanged { control_bar: false },
            0xB2 => InputEvent::ModeChanged { control_bar: true },
            0x39..=0x43 => {
                let x = if data.len() >= 13 {
                    (((data[11] as i32) << 8) | data[12] as i32) & 0xFFFF
                } else {
                    -1
                };
                InputEvent::BarTap { code: index, x }
            }
            1..=10 => InputEvent::Key {
                index,
                down: state == 1,
            },
            // 旋钮旋转编码（实测）：1=0xA0/0xA1，2=0x50/0x51，3=0x90/0x91，4=0x70/0x71
            0xA0 | 0xA1 => InputEvent::DialRotate { id: 1, dir: (index & 1) as i8 },
            0x50 | 0x51 => InputEvent::DialRotate { id: 2, dir: (index & 1) as i8 },
            0x90 | 0x91 => InputEvent::DialRotate { id: 3, dir: (index & 1) as i8 },
            0x70 | 0x71 => InputEvent::DialRotate { id: 4, dir: (index & 1) as i8 },
            // 旋钮按下编码（实测/协议文档）：0x37=1、0x35=2、0x33=3、0x36=4
            0x37 => InputEvent::DialPress { id: 1, down: state == 1 },
            0x35 => InputEvent::DialPress { id: 2, down: state == 1 },
            0x33 => InputEvent::DialPress { id: 3, down: state == 1 },
            0x36 => InputEvent::DialPress { id: 4, down: state == 1 },
            _ => InputEvent::Unknown { index, state },
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 解析按键按下与松开() {
        let mut buf = vec![0u8; 11];
        buf[..9].copy_from_slice(b"ACK\x00\x00OK\x00\x00");
        buf[9] = 1;
        buf[10] = 1;
        assert_eq!(parse(&buf), Some(InputEvent::Key { index: 1, down: true }));
        buf[10] = 0;
        assert_eq!(parse(&buf), Some(InputEvent::Key { index: 1, down: false }));
    }

    #[test]
    fn 解析触摸坐标() {
        let mut buf = vec![0u8; 14];
        buf[..7].copy_from_slice(b"ACK\x00ARX");
        buf[10] = 0x01;
        buf[11] = 0x15;
        buf[12] = 0x00;
        buf[13] = 0x3F;
        assert_eq!(
            parse(&buf),
            Some(InputEvent::TouchMove { x: 0x0115, y: 0x003F })
        );
    }

    #[test]
    fn 解析旋钮旋转与按下() {
        let mut buf = vec![0u8; 11];
        buf[..9].copy_from_slice(b"ACK\x00\x00OK\x00\x00");
        buf[9] = 0xA1;
        assert_eq!(parse(&buf), Some(InputEvent::DialRotate { id: 1, dir: 1 }));
        buf[9] = 0x50;
        assert_eq!(parse(&buf), Some(InputEvent::DialRotate { id: 2, dir: 0 }));
        buf[9] = 0x37;
        buf[10] = 1;
        assert_eq!(parse(&buf), Some(InputEvent::DialPress { id: 1, down: true }));
    }
}
