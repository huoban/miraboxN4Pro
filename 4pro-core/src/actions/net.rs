// 网络动作：HTTP、UDP、OSC（自实现 OSC 1.0 编码）

use crate::actions::{s_i64, s_str};
use serde_json::Value;
use std::net::UdpSocket;

fn host_port(settings: &Value) -> (String, u16) {
    let host = s_str(settings, &["host", "ip", "address", "IP"]).unwrap_or_else(|| "127.0.0.1".to_string());
    let port = s_i64(settings, &["port", "Port"]).unwrap_or(9000) as u16;
    (host, port)
}

pub fn http(settings: &Value, dry_run: bool) {
    let url = s_str(settings, &["url", "path", "URL"]).unwrap_or_default();
    if url.is_empty() {
        crate::log_warn!("HTTP 动作缺少 url");
        return;
    }
    let method = s_str(settings, &["method", "Method", "type"]).unwrap_or_else(|| "GET".to_string());
    let body = s_str(settings, &["body", "data", "Body"]).unwrap_or_default();
    if dry_run {
        crate::log_info!("[试运行] HTTP {} {}", method, url);
        return;
    }
    crate::log_info!("HTTP {} {}", method, url);
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(3))
        .build();
    let result = if method.eq_ignore_ascii_case("POST") {
        agent.post(&url).send_string(&body)
    } else {
        agent.get(&url).call()
    };
    match result {
        Ok(resp) => crate::log_info!("HTTP 完成，状态码 {}", resp.status()),
        Err(err) => crate::log_error!("HTTP 失败：{}", err),
    }
}

fn osc_pad(mut data: Vec<u8>) -> Vec<u8> {
    while data.len() % 4 != 0 {
        data.push(0);
    }
    data
}

fn osc_string(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    osc_pad(v)
}

/// 构建 OSC 消息：settings.address + settings.args（数字/字符串数组）
pub fn build_osc(address: &str, args: &[Value]) -> Vec<u8> {
    let mut packet = osc_string(address);
    let mut tags = String::from(",");
    let mut payload = Vec::new();
    for arg in args {
        if let Some(i) = arg.as_i64() {
            tags.push('i');
            payload.extend_from_slice(&(i as i32).to_be_bytes());
        } else if let Some(f) = arg.as_f64() {
            tags.push('f');
            payload.extend_from_slice(&(f as f32).to_be_bytes());
        } else if let Some(s) = arg.as_str() {
            tags.push('s');
            payload.extend(osc_string(s));
        }
    }
    packet.extend(osc_string(&tags));
    packet.extend(payload);
    packet
}

pub fn osc(settings: &Value, dry_run: bool) {
    let (host, port) = host_port(settings);
    let address = s_str(settings, &["address", "oscAddress", "path"]).unwrap_or_else(|| "/n4pro".to_string());
    let args: Vec<Value> = settings
        .get("args")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let packet = build_osc(&address, &args);
    if dry_run {
        crate::log_info!("[试运行] OSC {}:{} {} 字节", host, port, packet.len());
        return;
    }
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(sock) => match sock.send_to(&packet, format!("{}:{}", host, port)) {
            Ok(n) => crate::log_info!("OSC 已发送 {} 字节到 {}:{}", n, host, port),
            Err(e) => crate::log_error!("OSC 发送失败：{}", e),
        },
        Err(e) => crate::log_error!("UDP 绑定失败：{}", e),
    }
}

/// UDP 动作：若 address 以 "/" 开头或带 osc 标记，则按 OSC 发送
pub fn udp_or_osc(settings: &Value, dry_run: bool) {
    let is_osc = settings.get("osc").and_then(|v| v.as_bool()).unwrap_or(false)
        || settings
            .get("address")
            .and_then(|v| v.as_str())
            .map(|s| s.starts_with('/'))
            .unwrap_or(false);
    if is_osc {
        osc(settings, dry_run);
        return;
    }
    let (host, port) = host_port(settings);
    let payload = s_str(settings, &["message", "payload", "data", "content"]).unwrap_or_default();
    if dry_run {
        crate::log_info!("[试运行] UDP {}:{} 内容 {:?}", host, port, payload);
        return;
    }
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(sock) => match sock.send_to(payload.as_bytes(), format!("{}:{}", host, port)) {
            Ok(n) => crate::log_info!("UDP 已发送 {} 字节到 {}:{}", n, host, port),
            Err(e) => crate::log_error!("UDP 发送失败：{}", e),
        },
        Err(e) => crate::log_error!("UDP 绑定失败：{}", e),
    }
}
