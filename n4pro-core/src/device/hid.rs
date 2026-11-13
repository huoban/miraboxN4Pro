// HID 设备层：单句柄 + 重叠 I/O（与 hidapi 一致），读写互不阻塞
// 通道参数（实测）：VID 0x5548 / PID 0x1008 / usage_page 0xFFA0 / usage 1
// 输出 1025 字节（含 ReportID），输入 513 字节（含 ReportID）

use crate::platform::win::*;
use std::ffi::c_void;
use std::mem::size_of;

pub struct HidDevice {
    handle: HANDLE,
    read_event: HANDLE,
    write_event: HANDLE,
    pub output_len: usize,
    pub input_len: usize,
}

impl HidDevice {
    pub fn open_n4pro() -> Result<Self, String> {
        unsafe {
            let mut guid = GUID::default();
            HidD_GetHidGuid(&mut guid);
            let devs = SetupDiGetClassDevsW(
                &guid,
                std::ptr::null(),
                0,
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            );
            if devs == INVALID_HANDLE_VALUE {
                return Err("SetupDiGetClassDevsW 失败".to_string());
            }
            let mut found: Option<(HANDLE, u16, u16)> = None;
            let mut index = 0u32;
            loop {
                let mut iface = SP_DEVICE_INTERFACE_DATA {
                    cb_size: size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                    interface_class_guid: GUID::default(),
                    flags: 0,
                    reserved: 0,
                };
                if SetupDiEnumDeviceInterfaces(devs, std::ptr::null_mut(), &guid, index, &mut iface) == 0 {
                    break;
                }
                index += 1;

                let mut required = 0u32;
                SetupDiGetDeviceInterfaceDetailW(
                    devs,
                    &mut iface,
                    std::ptr::null_mut(),
                    0,
                    &mut required,
                    std::ptr::null_mut(),
                );
                if required == 0 {
                    continue;
                }
                let mut buf = vec![0u8; required as usize];
                let detail = buf.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
                (*detail).cb_size = if cfg!(target_pointer_width = "64") { 8 } else { 6 };
                if SetupDiGetDeviceInterfaceDetailW(
                    devs,
                    &mut iface,
                    detail,
                    required,
                    &mut required,
                    std::ptr::null_mut(),
                ) == 0
                {
                    continue;
                }
                let path_ptr = (*detail).device_path.as_ptr();
                let mut len = 0usize;
                while *path_ptr.add(len) != 0 {
                    len += 1;
                }
                let path = String::from_utf16_lossy(std::slice::from_raw_parts(path_ptr, len));
                let wide = to_wide(&path);

                let h = CreateFileW(
                    wide.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    std::ptr::null_mut(),
                    OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED,
                    0,
                );
                if h == INVALID_HANDLE_VALUE {
                    crate::log_warn!(
                        "HID 打开失败：{} 错误码 {}",
                        path,
                        GetLastError()
                    );
                    continue;
                }
                let mut attrs = HIDD_ATTRIBUTES {
                    size: size_of::<HIDD_ATTRIBUTES>() as u32,
                    ..Default::default()
                };
                if HidD_GetAttributes(h, &mut attrs) == 0
                    || attrs.vendor_id != 0x5548
                    || attrs.product_id != 0x1008
                {
                    CloseHandle(h);
                    continue;
                }
                let mut pp: *mut c_void = std::ptr::null_mut();
                if HidD_GetPreparsedData(h, &mut pp) == 0 {
                    CloseHandle(h);
                    continue;
                }
                let mut caps = HIDP_CAPS::default();
                let status = HidP_GetCaps(pp, &mut caps);
                HidD_FreePreparsedData(pp);
                crate::log_info!(
                    "HID 集合：UP=0x{:X} U={} OUT={} IN={}",
                    caps.usage_page,
                    caps.usage,
                    caps.output_report_byte_length,
                    caps.input_report_byte_length
                );
                if status < 0 || caps.usage_page != 0xFFA0 || caps.usage != 1 {
                    CloseHandle(h);
                    continue;
                }
                let _ = HidD_SetNumInputBuffers(h, 64);
                found = Some((h, caps.output_report_byte_length, caps.input_report_byte_length));
                break;
            }
            SetupDiDestroyDeviceInfoList(devs);

            match found {
                Some((handle, out_len, in_len)) => {
                    let read_event = CreateEventW(std::ptr::null_mut(), 0, 0, std::ptr::null());
                    let write_event = CreateEventW(std::ptr::null_mut(), 0, 0, std::ptr::null());
                    Ok(HidDevice {
                        handle,
                        read_event,
                        write_event,
                        output_len: if out_len == 0 { 1025 } else { out_len as usize },
                        input_len: if in_len == 0 { 513 } else { in_len as usize },
                    })
                }
                None => Err(
                    "未找到可用的 N4 Pro 厂商 HID 集合（VID 0x5548/PID 0x1008/usage 0xFFA0）"
                        .to_string(),
                ),
            }
        }
    }

    pub fn write_frame(&self, frame: &[u8]) -> Result<(), String> {
        unsafe {
            ResetEvent(self.write_event);
            let mut ov = OVERLAPPED {
                h_event: self.write_event,
                ..Default::default()
            };
            let mut written = 0u32;
            let ok = WriteFile(
                self.handle,
                frame.as_ptr(),
                frame.len() as u32,
                &mut written,
                &mut ov,
            );
            if ok != 0 {
                return Ok(());
            }
            let err = GetLastError();
            if err != ERROR_IO_PENDING {
                return Err(format!("HID 写入失败，错误码 {}", err));
            }
            let wait = WaitForSingleObject(self.write_event, 2000);
            if wait != 0 {
                return Err("HID 写入超时".to_string());
            }
            let mut transferred = 0u32;
            if GetOverlappedResult(self.handle, &mut ov, &mut transferred, 0) == 0 {
                return Err(format!(
                    "HID 写入完成失败，错误码 {}",
                    GetLastError()
                ));
            }
            if transferred as usize != frame.len() {
                return Err(format!("HID 写入不完整：{}/{} 字节", transferred, frame.len()));
            }
            Ok(())
        }
    }

    pub fn read_report(&self, buf: &mut [u8]) -> Result<usize, String> {
        unsafe {
            ResetEvent(self.read_event);
            let mut ov = OVERLAPPED {
                h_event: self.read_event,
                ..Default::default()
            };
            let mut read = 0u32;
            let ok = ReadFile(
                self.handle,
                buf.as_mut_ptr(),
                buf.len() as u32,
                &mut read,
                &mut ov,
            );
            if ok != 0 {
                return Ok(read as usize);
            }
            let err = GetLastError();
            if err != ERROR_IO_PENDING {
                return Err(format!("HID 读取失败，错误码 {}", err));
            }
            WaitForSingleObject(self.read_event, INFINITE);
            let mut transferred = 0u32;
            if GetOverlappedResult(self.handle, &mut ov, &mut transferred, 0) == 0 {
                return Err(format!(
                    "HID 读取完成失败，错误码 {}",
                    GetLastError()
                ));
            }
            Ok(transferred as usize)
        }
    }
}

impl Drop for HidDevice {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.read_event);
            CloseHandle(self.write_event);
            CloseHandle(self.handle);
        }
    }
}
