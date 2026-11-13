// 设备输出协议：1025 字节帧（ReportID 0 + 1024 载荷）
// 命令帧：载荷以 "CRT" 00 00 + 3 字节命令开头
// 图像帧：CRT 头之后紧跟 PNG 原始字节按 1024 分片，分片帧不再带任何头

pub const FRAME_LEN: usize = 1025;
pub const PAYLOAD_LEN: usize = 1024;
pub const REPORT_ID: u8 = 0x00;

const CMD_OFFSET: usize = 5;

fn frame_from(payload: [u8; PAYLOAD_LEN]) -> Vec<u8> {
    let mut frame = vec![0u8; FRAME_LEN];
    frame[0] = REPORT_ID;
    frame[1..].copy_from_slice(&payload);
    frame
}

fn header(payload: &mut [u8; PAYLOAD_LEN]) {
    payload[0] = b'C';
    payload[1] = b'R';
    payload[2] = b'T';
}

fn command(name: &str) -> Vec<u8> {
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in name.bytes().take(8).enumerate() {
        payload[CMD_OFFSET + i] = b;
    }
    frame_from(payload)
}

pub fn dis() -> Vec<u8> {
    command("DIS")
}

pub fn lig(brightness: u8) -> Vec<u8> {
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in b"LIG".iter().enumerate() {
        payload[CMD_OFFSET + i] = *b;
    }
    payload[10] = brightness.min(100);
    frame_from(payload)
}

/// 标签/灯带亮度：实测载荷为 "LBLIG" 后跟亮度值
pub fn lblig(brightness: u8) -> Vec<u8> {
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in b"LBLIG".iter().enumerate() {
        payload[CMD_OFFSET + i] = *b;
    }
    payload[10] = brightness.min(100);
    frame_from(payload)
}

/// 查询/握手命令，参数与实测一致
pub fn qucmd() -> Vec<u8> {
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in b"QUCMD".iter().enumerate() {
        payload[CMD_OFFSET + i] = *b;
    }
    payload[10] = 0x1F;
    payload[11] = 0x11;
    payload[13] = 0x11;
    payload[15] = 0x11;
    frame_from(payload)
}

pub fn bgcle(mode: u8) -> Vec<u8> {
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in b"BGCLE".iter().enumerate() {
        payload[CMD_OFFSET + i] = *b;
    }
    payload[10] = mode;
    frame_from(payload)
}

pub fn cle() -> Vec<u8> {
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in b"CLE".iter().enumerate() {
        payload[CMD_OFFSET + i] = *b;
    }
    payload[11] = 0xFF;
    frame_from(payload)
}

pub fn setlb() -> Vec<u8> {
    command("SETLB")
}

pub fn stp() -> Vec<u8> {
    command("STP")
}

pub fn connect() -> Vec<u8> {
    command("CONNECT")
}

/// 2 秒周期心跳，保持 USB 链路
pub fn rgbs() -> Vec<u8> {
    command("RGBS")
}

/// 把 PNG 字节切成 1024 字节载荷帧（无命令头）
fn chunk_frames(png: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    while offset < png.len() {
        let mut payload = [0u8; PAYLOAD_LEN];
        let n = (png.len() - offset).min(PAYLOAD_LEN);
        payload[..n].copy_from_slice(&png[offset..offset + n]);
        out.push(frame_from(payload));
        offset += n;
    }
    if out.is_empty() {
        out.push(frame_from([0u8; PAYLOAD_LEN]));
    }
    out
}

/// 触摸条图像（BGPIC）：800×130
pub fn bgpic_frames(png: &[u8], width: u16, height: u16) -> Vec<Vec<u8>> {
    let len = png.len() as u32;
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in b"BGPIC".iter().enumerate() {
        payload[CMD_OFFSET + i] = *b;
    }
    payload[10] = 0x00;
    payload[11] = ((len >> 16) & 0xFF) as u8;
    payload[12] = ((len >> 8) & 0xFF) as u8;
    payload[13] = (len & 0xFF) as u8;
    payload[18] = ((width >> 8) & 0xFF) as u8;
    payload[19] = (width & 0xFF) as u8;
    payload[20] = ((height >> 8) & 0xFF) as u8;
    payload[21] = (height & 0xFF) as u8;
    payload[22] = 0x00;
    payload[23] = 0xFF;

    let mut frames = vec![frame_from(payload)];
    frames.extend(chunk_frames(png));
    frames
}

/// 按键图像（BAT）：112×112，索引 6..15
pub fn bat_frames(png: &[u8], index: u8) -> Vec<Vec<u8>> {
    let len = (png.len().min(0xFFFF)) as u16;
    let mut payload = [0u8; PAYLOAD_LEN];
    header(&mut payload);
    for (i, b) in b"BAT".iter().enumerate() {
        payload[CMD_OFFSET + i] = *b;
    }
    payload[10] = ((len >> 8) & 0xFF) as u8;
    payload[11] = (len & 0xFF) as u8;
    payload[12] = index;

    let mut frames = vec![frame_from(payload)];
    frames.extend(chunk_frames(png));
    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 亮度帧结构正确() {
        let f = lig(60);
        assert_eq!(f.len(), FRAME_LEN);
        assert_eq!(f[0], REPORT_ID);
        assert_eq!(&f[1..4], b"CRT");
        assert_eq!(&f[6..9], b"LIG");
        assert_eq!(f[11], 60);
    }

    #[test]
    fn 清空帧带结束标记() {
        let f = cle();
        assert_eq!(&f[6..9], b"CLE");
        assert_eq!(f[12], 0xFF);
    }

    #[test]
    fn 触摸条图像分片与长度字段一致() {
        let data = vec![0xABu8; 3000];
        let frames = bgpic_frames(&data, 800, 130);
        // 1 个头帧 + ceil(3000/1024)=3 个数据帧
        assert_eq!(frames.len(), 4);
        let len = ((frames[0][12] as u32) << 16) | ((frames[0][13] as u32) << 8) | frames[0][14] as u32;
        assert_eq!(len, 3000);
        assert_eq!(frames[0][19], 0x03); // 宽 800 高字节
        assert_eq!(frames[0][20], 0x20); // 宽 800 低字节
        assert_eq!(frames[0][22], 130); // 高
        assert_eq!(&frames[1][1..5], &data[..4]);
    }

    #[test]
    fn 按键图像索引与长度正确() {
        let data = vec![0x11u8; 5000];
        let frames = bat_frames(&data, 11);
        assert_eq!(frames.len(), 1 + 5);
        assert_eq!(&frames[0][6..9], b"BAT");
        assert_eq!(frames[0][13], 11);
        let len = ((frames[0][11] as usize) << 8) | frames[0][12] as usize;
        assert_eq!(len, 5000);
    }
}
