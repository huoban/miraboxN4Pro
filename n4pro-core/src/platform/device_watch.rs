// USB 设备接入监听：隐藏窗口 + WM_DEVICECHANGE，阻塞等待，不占用 CPU
// 设备到达或 2 秒兜底轮询命中后返回；调用方重新打开设备

use crate::device::HidDevice;
use crate::platform::win::*;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

static ARRIVED: AtomicBool = AtomicBool::new(false);
static CLASS_ONCE: Once = Once::new();

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    if msg == WM_DEVICECHANGE && w == DBT_DEVICEARRIVAL {
        ARRIVED.store(true, Ordering::SeqCst);
        return 1;
    }
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

/// 阻塞等待 N4 Pro 接入（零 CPU 占用）
pub fn wait_for_device() {
    unsafe {
        let class_name = to_wide("N4ProDeviceWatch");
        let instance = GetModuleHandleW(std::ptr::null());
        CLASS_ONCE.call_once(|| {
            let wc = WNDCLASSW {
                style: 0,
                lpfn_wnd_proc: Some(wnd_proc),
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: instance,
                h_icon: 0,
                h_cursor: 0,
                hbr_background: 0,
                lpsz_menu_name: std::ptr::null(),
                lpsz_class_name: class_name.as_ptr(),
            };
            RegisterClassW(&wc);
        });

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            class_name.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            0,
            0,
            instance,
            std::ptr::null_mut(),
        );

        let mut hid_guid = GUID::default();
        HidD_GetHidGuid(&mut hid_guid);
        let mut filter = DEV_BROADCAST_DEVICEINTERFACE_W {
            size: std::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as u32,
            device_type: DBT_DEVTYP_DEVICEINTERFACE,
            reserved: 0,
            class_guid: hid_guid,
            name: [0],
        };
        let notify = if hwnd != 0 {
            RegisterDeviceNotificationW(hwnd, &mut filter as *mut _ as *mut c_void, 0)
        } else {
            0
        };

        crate::log_info!("等待 USB 设备接入…");
        // 避免设备仍在但打不开时形成忙循环
        std::thread::sleep(std::time::Duration::from_millis(500));
        loop {
            if HidDevice::is_present() {
                break;
            }
            // 最多阻塞 2 秒；有窗口消息（含设备到达）会立即返回
            let r = MsgWaitForMultipleObjects(0, std::ptr::null(), 0, 2000, QS_ALLINPUT);
            if r == 1 {
                let mut msg = MSG::default();
                while PeekMessageW(&mut msg, 0, 0, 0, 1) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            if ARRIVED.swap(false, Ordering::SeqCst) {
                // 设备到达消息已收到，再确认一次可打开
                if HidDevice::is_present() {
                    break;
                }
            }
        }

        if notify != 0 {
            UnregisterDeviceNotification(notify);
        }
        if hwnd != 0 {
            DestroyWindow(hwnd);
        }
        crate::log_info!("检测到设备接入，开始重新初始化");
    }
}
