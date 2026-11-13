// 切换音频设备：Core Audio（IMMDeviceEnumerator + IPolicyConfig）
// 官方设置键：radio、device、select、select2（设备 ID）
// radio=0 时在 select 与 select2 之间切换；radio!=0 时固定切到 select

use crate::actions::{s_i64, s_str};
use crate::platform::win::{to_wide, GUID};
use serde_json::Value;
use std::ffi::c_void;

const CLSCTX_ALL: u32 = 0x17;
const COINIT_APARTMENTTHREADED: u32 = 0x2;

const CLSID_MM_DEVICE_ENUMERATOR: GUID = GUID {
    data1: 0xBCDE0395,
    data2: 0xE52F,
    data3: 0x467C,
    data4: [0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E],
};
const IID_IMM_DEVICE_ENUMERATOR: GUID = GUID {
    data1: 0xA95664D2,
    data2: 0x9614,
    data3: 0x4F35,
    data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
};
const CLSID_POLICY_CONFIG_CLIENT: GUID = GUID {
    data1: 0x870AF99C,
    data2: 0x171D,
    data3: 0x4F9E,
    data4: [0xAF, 0x0D, 0xE6, 0x3D, 0xF4, 0x0C, 0x2B, 0xC9],
};
const IID_IPOLICY_CONFIG: GUID = GUID {
    data1: 0xF8679F50,
    data2: 0x850A,
    data3: 0x41CF,
    data4: [0x9C, 0x72, 0x43, 0x0F, 0x29, 0x02, 0x90, 0xC8],
};

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
    fn CoCreateInstance(
        clsid: *const GUID,
        outer: *mut c_void,
        clsctx: u32,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> i32;
    fn CoUninitialize();
    fn CoTaskMemFree(p: *mut c_void);
}

type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetDefaultFn = unsafe extern "system" fn(*mut c_void, i32, i32, *mut *mut c_void) -> i32;
type GetIdFn = unsafe extern "system" fn(*mut c_void, *mut *mut u16) -> i32;
type SetDefaultFn = unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32;

unsafe fn vtable(this: *mut c_void) -> *mut *mut c_void {
    *(this as *mut *mut *mut c_void)
}

unsafe fn release(this: *mut c_void) {
    if this.is_null() {
        return;
    }
    let f: ReleaseFn = std::mem::transmute(*vtable(this).add(2));
    f(this);
}

/// 读取设备 ID（CoTaskMem 字符串）
unsafe fn device_id(device: *mut c_void) -> Option<String> {
    if device.is_null() {
        return None;
    }
    let f: GetIdFn = std::mem::transmute(*vtable(device).add(5));
    let mut ptr: *mut u16 = std::ptr::null_mut();
    let hr = f(device, &mut ptr);
    if hr < 0 || ptr.is_null() {
        return None;
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
    CoTaskMemFree(ptr as *mut c_void);
    Some(text)
}

/// 读取当前默认设备 ID
fn default_device_id(flow: i32) -> Option<String> {
    unsafe {
        let mut enumerator: *mut c_void = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_MM_DEVICE_ENUMERATOR,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IID_IMM_DEVICE_ENUMERATOR,
            &mut enumerator,
        );
        if hr < 0 || enumerator.is_null() {
            return None;
        }
        let f: GetDefaultFn = std::mem::transmute(*vtable(enumerator).add(4));
        let mut device: *mut c_void = std::ptr::null_mut();
        let hr = f(enumerator, flow, 0, &mut device);
        release(enumerator);
        if hr < 0 {
            return None;
        }
        let id = device_id(device);
        release(device);
        id
    }
}

/// 设置默认设备（控制台/多媒体/通信三个角色）
fn set_default_device(id: &str) -> bool {
    unsafe {
        let mut policy: *mut c_void = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_POLICY_CONFIG_CLIENT,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IID_IPOLICY_CONFIG,
            &mut policy,
        );
        if hr < 0 || policy.is_null() {
            return false;
        }
        let f: SetDefaultFn = std::mem::transmute(*vtable(policy).add(13));
        let wide = to_wide(id);
        let mut ok = false;
        for role in 0..3 {
            if f(policy, wide.as_ptr(), role) >= 0 {
                ok = true;
            }
        }
        release(policy);
        ok
    }
}

/// 执行切换
pub fn switch(settings: &Value, dry_run: bool) {
    // 数据流：0=播放 1=录音
    let flow = match s_i64(settings, &["device", "Device"]).unwrap_or(0) {
        1 => 1,
        _ => match s_str(settings, &["device", "Device"]).map(|s| s.to_lowercase()) {
            Some(s) if s.contains("record") || s.contains("capture") => 1,
            _ => 0,
        },
    };
    let radio = s_i64(settings, &["radio"]).unwrap_or(0);
    let select = s_str(settings, &["select", "deviceId", "DeviceId"]).unwrap_or_default();
    let select2 = s_str(settings, &["select2", "deviceId2", "DeviceId2"]).unwrap_or_default();

    let mut target = select.clone();
    if radio == 0 && !select2.is_empty() {
        let current = default_device_id(flow).unwrap_or_default();
        target = if current.eq_ignore_ascii_case(&select) {
            select2.clone()
        } else {
            select.clone()
        };
    }
    if target.is_empty() {
        crate::log_warn!("切换音频设备缺少目标设备 ID");
        return;
    }
    if dry_run {
        crate::log_info!(
            "[试运行] 切换音频设备：{}（数据流 {}，radio {}）",
            target,
            if flow == 0 { "播放" } else { "录音" },
            radio
        );
        return;
    }

    let hr = unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED) };
    let ok = set_default_device(&target);
    if hr >= 0 {
        unsafe { CoUninitialize() };
    }
    if ok {
        crate::log_info!(
            "已切换默认{}设备：{}",
            if flow == 0 { "播放" } else { "录音" },
            target
        );
    } else {
        crate::log_warn!("切换音频设备失败：{}", target);
    }
}
