// Windows 平台层：HID、输入模拟、GDI 文字、进程与路径等最小 Win32 封装
// 说明：全部使用系统 DLL 直接声明，避免引入额外 C 依赖，保证单 exe 可编译。

#![allow(non_snake_case)]

use std::ffi::c_void;
use std::path::PathBuf;

pub type HANDLE = isize;
pub type HWND = isize;
pub type HDC = isize;
pub type HBITMAP = isize;
pub type HGDIOBJ = isize;
pub type HFONT = isize;
pub type BOOL = i32;
pub type DWORD = u32;
pub type UINT = u32;
pub type WORD = u16;
pub type LONG = i32;
pub type ULONG_PTR = usize;
pub type COLORREF = u32;
pub type NTSTATUS = i32;
pub type BOOLEAN = u8;

pub const INVALID_HANDLE_VALUE: HANDLE = -1;
pub const GENERIC_READ: u32 = 0x8000_0000;
pub const GENERIC_WRITE: u32 = 0x4000_0000;
pub const FILE_SHARE_READ: u32 = 0x0000_0001;
pub const FILE_SHARE_WRITE: u32 = 0x0000_0002;
pub const OPEN_EXISTING: u32 = 3;
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
pub const ERROR_ALREADY_EXISTS: u32 = 183;
pub const DIGCF_PRESENT: u32 = 0x0000_0002;
pub const DIGCF_DEVICEINTERFACE: u32 = 0x0000_0010;
pub const KEYEVENTF_KEYUP: u32 = 0x0002;
pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
pub const DT_LEFT: u32 = 0x0000;
pub const DT_CENTER: u32 = 0x0001;
pub const DT_RIGHT: u32 = 0x0002;
pub const DT_VCENTER: u32 = 0x0004;
pub const DT_TOP: u32 = 0x0000;
pub const DT_BOTTOM: u32 = 0x0008;
pub const DT_SINGLELINE: u32 = 0x0020;
pub const DT_END_ELLIPSIS: u32 = 0x0000_8000;
pub const DT_WORDBREAK: u32 = 0x0010;
pub const DT_NOCLIP: u32 = 0x0000_0100;
pub const TRANSPARENT: i32 = 1;
pub const ANTIALIASED_QUALITY: u32 = 4;
pub const DEFAULT_CHARSET: u32 = 1;
pub const SW_HIDE: i32 = 0;
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const STARTF_USESHOWWINDOW: u32 = 0x0000_0001;
pub const FILE_FLAG_OVERLAPPED: u32 = 0x4000_0000;
pub const ERROR_IO_PENDING: u32 = 997;
pub const INFINITE: u32 = 0xFFFF_FFFF;

// 输入模拟（SendInput）相关常量
pub const INPUT_MOUSE: u32 = 0;
pub const INPUT_KEYBOARD: u32 = 1;
pub const MOUSEEVENTF_MOVE: u32 = 0x0001;
pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
pub const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
pub const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
pub const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
pub const MOUSEEVENTF_XDOWN: u32 = 0x0080;
pub const MOUSEEVENTF_XUP: u32 = 0x0100;
pub const MOUSEEVENTF_WHEEL: u32 = 0x0800;
pub const MOUSEEVENTF_HWHEEL: u32 = 0x1000;
pub const XBUTTON1: u32 = 0x0001;
pub const XBUTTON2: u32 = 0x0002;
pub const KEYEVENTF_SCANCODE: u32 = 0x0008;

#[repr(C)]
#[derive(Default)]
pub struct OVERLAPPED {
    pub internal: usize,
    pub internal_high: usize,
    pub offset: u32,
    pub offset_high: u32,
    pub h_event: HANDLE,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[repr(C)]
pub struct SP_DEVICE_INTERFACE_DATA {
    pub cb_size: u32,
    pub interface_class_guid: GUID,
    pub flags: u32,
    pub reserved: usize,
}

#[repr(C)]
pub struct SP_DEVICE_INTERFACE_DETAIL_DATA_W {
    pub cb_size: u32,
    pub device_path: [u16; 1],
}

#[repr(C)]
#[derive(Default)]
pub struct HIDD_ATTRIBUTES {
    pub size: u32,
    pub vendor_id: u16,
    pub product_id: u16,
    pub version_number: u16,
}

#[repr(C)]
#[derive(Default)]
pub struct HIDP_CAPS {
    pub usage: u16,
    pub usage_page: u16,
    pub input_report_byte_length: u16,
    pub output_report_byte_length: u16,
    pub feature_report_byte_length: u16,
    pub reserved: [u16; 17],
    pub number_link_collection_nodes: u16,
    pub number_input_button_caps: u16,
    pub number_input_value_caps: u16,
    pub number_input_data_indices: u16,
    pub number_output_button_caps: u16,
    pub number_output_value_caps: u16,
    pub number_output_data_indices: u16,
    pub number_feature_button_caps: u16,
    pub number_feature_value_caps: u16,
    pub number_feature_data_indices: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct KEYBDINPUT {
    pub w_vk: WORD,
    pub w_scan: WORD,
    pub dw_flags: DWORD,
    pub time: DWORD,
    pub dw_extra_info: ULONG_PTR,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MOUSEINPUT {
    pub dx: LONG,
    pub dy: LONG,
    pub mouse_data: DWORD,
    pub dw_flags: DWORD,
    pub time: DWORD,
    pub dw_extra_info: ULONG_PTR,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union INPUT_U {
    pub ki: KEYBDINPUT,
    pub mi: MOUSEINPUT,
    pub pad: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct INPUT {
    pub r#type: DWORD,
    pub u: INPUT_U,
}

#[repr(C)]
#[derive(Default)]
pub struct SYSTEMTIME {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

#[repr(C)]
#[derive(Default)]
pub struct BITMAPINFOHEADER {
    pub bi_size: u32,
    pub bi_width: i32,
    pub bi_height: i32,
    pub bi_planes: u16,
    pub bi_bit_count: u16,
    pub bi_compression: u32,
    pub bi_size_image: u32,
    pub bi_xpels_per_meter: i32,
    pub bi_ypels_per_meter: i32,
    pub bi_clr_used: u32,
    pub bi_clr_important: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct BITMAPINFO {
    pub bmi_header: BITMAPINFOHEADER,
    pub bmi_colors: [u32; 1],
}

#[repr(C)]
pub struct STARTUPINFOW {
    pub cb: u32,
    pub lp_reserved: *mut u16,
    pub lp_desktop: *mut u16,
    pub lp_title: *mut u16,
    pub dw_x: u32,
    pub dw_y: u32,
    pub dw_x_size: u32,
    pub dw_y_size: u32,
    pub dw_x_count_chars: u32,
    pub dw_y_count_chars: u32,
    pub dw_fill_attribute: u32,
    pub dw_flags: u32,
    pub w_show_window: u16,
    pub cb_reserved2: u16,
    pub lp_reserved2: *mut u8,
    pub h_std_input: HANDLE,
    pub h_std_output: HANDLE,
    pub h_std_error: HANDLE,
}

impl Default for STARTUPINFOW {
    fn default() -> Self {
        Self {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            lp_reserved: std::ptr::null_mut(),
            lp_desktop: std::ptr::null_mut(),
            lp_title: std::ptr::null_mut(),
            dw_x: 0,
            dw_y: 0,
            dw_x_size: 0,
            dw_y_size: 0,
            dw_x_count_chars: 0,
            dw_y_count_chars: 0,
            dw_fill_attribute: 0,
            dw_flags: 0,
            w_show_window: 0,
            cb_reserved2: 0,
            lp_reserved2: std::ptr::null_mut(),
            h_std_input: 0,
            h_std_output: 0,
            h_std_error: 0,
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct PROCESS_INFORMATION {
    pub h_process: HANDLE,
    pub h_thread: HANDLE,
    pub dw_process_id: u32,
    pub dw_thread_id: u32,
}

#[link(name = "kernel32")]
extern "system" {
    pub fn CreateFileW(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: *mut c_void,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: HANDLE,
    ) -> HANDLE;
    pub fn ReadFile(h: HANDLE, buf: *mut u8, n: u32, read: *mut u32, ov: *mut OVERLAPPED) -> BOOL;
    pub fn WriteFile(h: HANDLE, buf: *const u8, n: u32, written: *mut u32, ov: *mut OVERLAPPED) -> BOOL;
    pub fn CloseHandle(h: HANDLE) -> BOOL;
    pub fn CreateMutexW(attrs: *mut c_void, initial: BOOL, name: *const u16) -> HANDLE;
    pub fn GetLastError() -> u32;
    pub fn FreeConsole() -> BOOL;
    pub fn GetConsoleWindow() -> HWND;
    pub fn Sleep(ms: u32);
    pub fn GetModuleFileNameW(h: HANDLE, buf: *mut u16, n: u32) -> u32;
    pub fn GetLocalTime(st: *mut SYSTEMTIME);
    pub fn CreateEventW(attrs: *mut c_void, manual: BOOL, initial: BOOL, name: *const u16) -> HANDLE;
    pub fn ResetEvent(h: HANDLE) -> BOOL;
    pub fn WaitForSingleObject(h: HANDLE, ms: u32) -> u32;
    pub fn GetOverlappedResult(
        h: HANDLE,
        ov: *mut OVERLAPPED,
        transferred: *mut u32,
        wait: BOOL,
    ) -> BOOL;
    pub fn CreateProcessW(
        app_name: *const u16,
        cmd_line: *mut u16,
        proc_attrs: *mut c_void,
        thread_attrs: *mut c_void,
        inherit: BOOL,
        creation_flags: u32,
        env: *mut c_void,
        cwd: *const u16,
        startup: *mut STARTUPINFOW,
        info: *mut PROCESS_INFORMATION,
    ) -> BOOL;
}

#[link(name = "setupapi")]
extern "system" {
    pub fn SetupDiGetClassDevsW(
        class_guid: *const GUID,
        enumerator: *const u16,
        parent: HWND,
        flags: u32,
    ) -> HANDLE;
    pub fn SetupDiEnumDeviceInterfaces(
        devs: HANDLE,
        dev_info: *mut c_void,
        interface_class_guid: *const GUID,
        index: u32,
        interface_data: *mut SP_DEVICE_INTERFACE_DATA,
    ) -> BOOL;
    pub fn SetupDiGetDeviceInterfaceDetailW(
        devs: HANDLE,
        interface_data: *mut SP_DEVICE_INTERFACE_DATA,
        detail: *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W,
        detail_size: u32,
        required: *mut u32,
        dev_info: *mut c_void,
    ) -> BOOL;
    pub fn SetupDiDestroyDeviceInfoList(devs: HANDLE) -> BOOL;
}

#[link(name = "hid")]
extern "system" {
    pub fn HidD_GetHidGuid(guid: *mut GUID);
    pub fn HidD_GetAttributes(h: HANDLE, attrs: *mut HIDD_ATTRIBUTES) -> BOOL;
    pub fn HidD_GetPreparsedData(h: HANDLE, data: *mut *mut c_void) -> BOOL;
    pub fn HidD_FreePreparsedData(data: *mut c_void) -> BOOL;
    pub fn HidP_GetCaps(data: *mut c_void, caps: *mut HIDP_CAPS) -> NTSTATUS;
    pub fn HidD_SetNumInputBuffers(h: HANDLE, num: u32) -> BOOL;
}

#[link(name = "user32")]
extern "system" {
    pub fn SendInput(n: u32, inputs: *const INPUT, size: i32) -> u32;
    pub fn keybd_event(vk: u8, scan: u8, flags: u32, extra: usize);
    pub fn SetCursorPos(x: i32, y: i32) -> BOOL;
    pub fn OpenClipboard(hwnd: HWND) -> BOOL;
    pub fn EmptyClipboard() -> BOOL;
    pub fn SetClipboardData(format: u32, mem: HANDLE) -> HANDLE;
    pub fn CloseClipboard() -> BOOL;
    pub fn MapVirtualKeyW(code: u32, map_type: u32) -> u32;
    pub fn SetProcessDPIAware() -> BOOL;
    pub fn DrawTextW(hdc: HDC, text: *const u16, len: i32, rect: *mut RECT, flags: u32) -> i32;
    pub fn CreateFontW(
        height: i32,
        width: i32,
        esc: i32,
        orient: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike: u32,
        charset: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch: u32,
        face: *const u16,
    ) -> HFONT;
    pub fn GetDC(hwnd: HWND) -> HDC;
    pub fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32;
    pub fn ShowWindow(hwnd: HWND, cmd: i32) -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateDIBSection(
        hdc: HDC,
        bmi: *const BITMAPINFO,
        usage: u32,
        bits: *mut *mut c_void,
        section: HANDLE,
        offset: u32,
    ) -> HBITMAP;
    pub fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(obj: HGDIOBJ) -> BOOL;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
}

#[link(name = "shell32")]
extern "system" {
    pub fn ShellExecuteW(
        hwnd: HWND,
        op: *const u16,
        file: *const u16,
        params: *const u16,
        dir: *const u16,
        show: i32,
    ) -> HANDLE;
}

/// 字符串转 UTF-16（带结尾 0）
pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 取 exe 所在目录
pub fn exe_dir() -> PathBuf {
    let mut buf = vec![0u16; 1024];
    let n = unsafe { GetModuleFileNameW(0, buf.as_mut_ptr(), buf.len() as u32) };
    if n == 0 {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }
    let s = String::from_utf16_lossy(&buf[..n as usize]);
    PathBuf::from(s)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 本地时间字符串 YYYY-MM-DD HH:mm:ss.SSS
pub fn now_text() -> String {
    let mut st = SYSTEMTIME::default();
    unsafe { GetLocalTime(&mut st) };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        st.year, st.month, st.day, st.hour, st.minute, st.second, st.milliseconds
    )
}

/// 修饰键状态（区分左右）
#[derive(Clone, Copy, Default)]
pub struct Mods {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
    pub r_ctrl: bool,
    pub r_shift: bool,
    pub r_alt: bool,
    pub r_win: bool,
}

const VK_LSHIFT: u16 = 0xA0;
const VK_RSHIFT: u16 = 0xA1;
const VK_LCONTROL: u16 = 0xA2;
const VK_RCONTROL: u16 = 0xA3;
const VK_LMENU: u16 = 0xA4;
const VK_RMENU: u16 = 0xA5;
const VK_LWIN: u16 = 0x5B;
const VK_RWIN: u16 = 0x5C;

/// 需要扩展键标志的虚拟键
fn is_extended_key(vk: u16) -> bool {
    matches!(vk, 0x21..=0x2E | 0x5B..=0x5D | 0x6F | 0x90 | 0xA3 | 0xA5)
}

/// 以扫描码方式发送单键（按下/抬起），兼容游戏与普通窗口
fn send_key_scan(vk: u16, up: bool) {
    let scan = unsafe { MapVirtualKeyW(vk as u32, 0) } as u16;
    let mut flags = KEYEVENTF_SCANCODE;
    if is_extended_key(vk) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_U {
            ki: KEYBDINPUT {
                w_vk: 0,
                w_scan: scan,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

/// 发送组合键：左右修饰键 + 多个主键（同时按下后逆序释放）
pub fn send_hotkey(vks: &[u16], mods: Mods) {
    let mut modifier_vks: Vec<u16> = Vec::new();
    if mods.ctrl {
        modifier_vks.push(VK_LCONTROL);
    }
    if mods.r_ctrl {
        modifier_vks.push(VK_RCONTROL);
    }
    if mods.shift {
        modifier_vks.push(VK_LSHIFT);
    }
    if mods.r_shift {
        modifier_vks.push(VK_RSHIFT);
    }
    if mods.alt {
        modifier_vks.push(VK_LMENU);
    }
    if mods.r_alt {
        modifier_vks.push(VK_RMENU);
    }
    if mods.win {
        modifier_vks.push(VK_LWIN);
    }
    if mods.r_win {
        modifier_vks.push(VK_RWIN);
    }

    for vk in &modifier_vks {
        send_key_scan(*vk, false);
        std::thread::sleep(std::time::Duration::from_millis(18));
    }
    for vk in vks {
        if *vk != 0 {
            send_key_scan(*vk, false);
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    }
    for vk in vks.iter().rev() {
        if *vk != 0 {
            send_key_scan(*vk, true);
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    }
    for vk in modifier_vks.iter().rev() {
        send_key_scan(*vk, true);
        std::thread::sleep(std::time::Duration::from_millis(18));
    }
}

/// 发送一次键盘按键（默认左修饰键）
pub fn key_press(vk: u16, ctrl: bool, shift: bool, alt: bool, win: bool) {
    send_hotkey(
        &[vk],
        Mods {
            ctrl,
            shift,
            alt,
            win,
            ..Default::default()
        },
    );
}

/// 直接发送媒体/音量键（按下+抬起）
pub fn media_key(vk: u16) {
    unsafe {
        keybd_event(vk as u8, 0, KEYEVENTF_EXTENDEDKEY, 0);
        keybd_event(vk as u8, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
    }
}

/// 绝对坐标移动鼠标（像素）
pub fn mouse_move_to(x: i32, y: i32) {
    unsafe {
        SetCursorPos(x, y);
    }
}

/// 鼠标按键：0=左 1=右 2=中 3=侧键1 4=侧键2
pub fn mouse_button(button: i32, down: bool) {
    let (flag, data) = match button {
        0 => (if down { MOUSEEVENTF_LEFTDOWN } else { MOUSEEVENTF_LEFTUP }, 0),
        1 => (if down { MOUSEEVENTF_RIGHTDOWN } else { MOUSEEVENTF_RIGHTUP }, 0),
        2 => (if down { MOUSEEVENTF_MIDDLEDOWN } else { MOUSEEVENTF_MIDDLEUP }, 0),
        3 => (if down { MOUSEEVENTF_XDOWN } else { MOUSEEVENTF_XUP }, XBUTTON1),
        4 => (if down { MOUSEEVENTF_XDOWN } else { MOUSEEVENTF_XUP }, XBUTTON2),
        _ => return,
    };
    let input = INPUT {
        r#type: INPUT_MOUSE,
        u: INPUT_U {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouse_data: data,
                dw_flags: flag,
                time: 0,
                dw_extra_info: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

/// 滚轮：horizontal=true 为横向滚动
pub fn mouse_wheel(amount: i32, horizontal: bool) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        u: INPUT_U {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouse_data: amount as u32,
                dw_flags: if horizontal { MOUSEEVENTF_HWHEEL } else { MOUSEEVENTF_WHEEL },
                time: 0,
                dw_extra_info: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

/// 写入 Unicode 文本到剪贴板
pub fn clipboard_set_text(text: &str) -> bool {
    unsafe {
        if OpenClipboard(0) == 0 {
            return false;
        }
        EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let mem = GlobalAlloc(0x0002, wide.len() * 2); // GMEM_MOVEABLE
        if mem == 0 {
            CloseClipboard();
            return false;
        }
        let ptr = GlobalLock(mem) as *mut u16;
        if ptr.is_null() {
            GlobalFree(mem);
            CloseClipboard();
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(mem);
        SetClipboardData(13, mem); // CF_UNICODETEXT
        CloseClipboard();
        true
    }
}

/// 扫描码方式模拟按键（Super Hotkeys：更接近物理键盘）
pub fn key_press_scancode(vk: u16, ctrl: bool, shift: bool, alt: bool, win: bool) {
    // 统一使用扫描码实现（见 send_hotkey）
    key_press(vk, ctrl, shift, alt, win);
}

/// 关闭指定路径对应的进程（force=true 强制结束）
pub fn close_app(path: &str, force: bool) -> bool {
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let cmd = if force {
        format!("taskkill /F /IM \"{}\"", name)
    } else {
        format!("taskkill /IM \"{}\"", name)
    };
    run_hidden(&cmd)
}

#[link(name = "PowrProf")]
extern "system" {
    fn SetSuspendState(hibernate: BOOLEAN, force: BOOLEAN, disable_wake: BOOLEAN) -> BOOLEAN;
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> HANDLE;
    fn GlobalLock(mem: HANDLE) -> *mut c_void;
    fn GlobalUnlock(mem: HANDLE) -> BOOL;
    fn GlobalFree(mem: HANDLE) -> HANDLE;
}

/// 系统睡眠（支持休眠时进入休眠）
pub fn suspend_pc() {
    unsafe {
        SetSuspendState(0, 0, 0);
    }
}

/// 开启 DPI 感知，保证鼠标绝对坐标与官方一致
pub fn set_dpi_aware() {
    unsafe {
        SetProcessDPIAware();
    }
}

// ---------------- USB 设备接入监听（隐藏窗口 + WM_DEVICECHANGE） ----------------

pub const WM_DEVICECHANGE: u32 = 0x0219;
pub const DBT_DEVICEARRIVAL: usize = 0x8000;
pub const DBT_DEVTYP_DEVICEINTERFACE: u32 = 5;
pub const QS_ALLINPUT: u32 = 0x04FF;
pub const WS_POPUP: u32 = 0x8000_0000;
pub const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;

pub type WNDPROC = extern "system" fn(HWND, u32, usize, isize) -> isize;

#[repr(C)]
pub struct WNDCLASSW {
    pub style: u32,
    pub lpfn_wnd_proc: Option<WNDPROC>,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: HANDLE,
    pub h_icon: HANDLE,
    pub h_cursor: HANDLE,
    pub hbr_background: HANDLE,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
}

#[repr(C)]
#[derive(Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Default)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub w_param: usize,
    pub l_param: isize,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
pub struct DEV_BROADCAST_DEVICEINTERFACE_W {
    pub size: u32,
    pub device_type: u32,
    pub reserved: u32,
    pub class_guid: GUID,
    pub name: [u16; 1],
}

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassW(class: *const WNDCLASSW) -> u16;
    pub fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: HWND,
        menu: HANDLE,
        instance: HANDLE,
        param: *mut c_void,
    ) -> HWND;
    pub fn DestroyWindow(hwnd: HWND) -> BOOL;
    pub fn DefWindowProcW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    pub fn RegisterDeviceNotificationW(h: HANDLE, filter: *mut c_void, flags: u32) -> HANDLE;
    pub fn UnregisterDeviceNotification(h: HANDLE) -> BOOL;
    pub fn MsgWaitForMultipleObjects(
        count: u32,
        handles: *const HANDLE,
        wait_all: BOOL,
        ms: u32,
        wake_mask: u32,
    ) -> u32;
    pub fn PeekMessageW(msg: *mut MSG, hwnd: HWND, min: u32, max: u32, remove: u32) -> BOOL;
    pub fn TranslateMessage(msg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(msg: *const MSG) -> isize;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HANDLE;
}

// ---------------- 控制台与系统托盘 ----------------

pub const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
pub const SW_RESTORE: i32 = 9;
pub const WM_APP: u32 = 0x8000;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_TIMER: u32 = 0x0113;
pub const NIF_MESSAGE: u32 = 0x0000_0001;
pub const NIF_ICON: u32 = 0x0000_0002;
pub const NIF_TIP: u32 = 0x0000_0004;
pub const NIM_ADD: u32 = 0x0000_0000;
pub const NIM_DELETE: u32 = 0x0000_0002;
pub const TPM_RIGHTBUTTON: u32 = 0x0002;
pub const TPM_RETURNCMD: u32 = 0x0100;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct COORD {
    pub x: i16,
    pub y: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SMALL_RECT {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CONSOLE_SCREEN_BUFFER_INFO {
    pub size: COORD,
    pub cursor_position: COORD,
    pub attributes: u16,
    pub window: SMALL_RECT,
    pub maximum_window_size: COORD,
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cb_size: u32,
    pub hwnd: HWND,
    pub u_id: u32,
    pub u_flags: u32,
    pub u_callback_message: u32,
    pub h_icon: HANDLE,
    pub sz_tip: [u16; 128],
    pub dw_state: u32,
    pub dw_state_mask: u32,
    pub sz_info: [u16; 256],
    pub u_version: u32,
    pub sz_info_title: [u16; 64],
    pub dw_info_flags: u32,
    pub guid_item: GUID,
    pub h_balloon_icon: HANDLE,
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetStdHandle(kind: u32) -> HANDLE;
    pub fn GetConsoleScreenBufferInfo(h: HANDLE, info: *mut CONSOLE_SCREEN_BUFFER_INFO) -> BOOL;
    pub fn SetConsoleScreenBufferSize(h: HANDLE, size: COORD) -> BOOL;
    pub fn SetConsoleWindowInfo(h: HANDLE, absolute: BOOL, rect: *const SMALL_RECT) -> BOOL;
}

#[link(name = "user32")]
extern "system" {
    pub fn IsIconic(hwnd: HWND) -> BOOL;
    pub fn SetForegroundWindow(hwnd: HWND) -> BOOL;
    pub fn LoadIconW(instance: HANDLE, name: *const u16) -> HANDLE;
    pub fn Shell_NotifyIconW(op: u32, data: *mut NOTIFYICONDATAW) -> BOOL;
    pub fn CreatePopupMenu() -> HANDLE;
    pub fn AppendMenuW(menu: HANDLE, flags: u32, id: usize, text: *const u16) -> BOOL;
    pub fn DestroyMenu(menu: HANDLE) -> BOOL;
    pub fn TrackPopupMenu(
        menu: HANDLE,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: HWND,
        rect: *const RECT,
    ) -> BOOL;
    pub fn GetCursorPos(p: *mut POINT) -> BOOL;
    pub fn SetTimer(hwnd: HWND, id: usize, ms: u32, proc: usize) -> usize;
    pub fn KillTimer(hwnd: HWND, id: usize) -> BOOL;
    pub fn GetMessageW(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> BOOL;
}

/// 控制台缓冲区高度固定为指定行数（旧日志自动滚掉）
pub fn setup_console(height: i16) {
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        if h == 0 || h == INVALID_HANDLE_VALUE {
            return;
        }
        let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
        if GetConsoleScreenBufferInfo(h, &mut info) == 0 {
            return;
        }
        let width = info.size.x.max(80);
        let window_height = info.window.bottom - info.window.top + 1;
        // 先把窗口高度收进目标缓冲区，再设置缓冲区高度
        if window_height > height {
            let rect = SMALL_RECT {
                left: 0,
                top: 0,
                right: width - 1,
                bottom: height - 1,
            };
            SetConsoleWindowInfo(h, 1, &rect);
        }
        SetConsoleScreenBufferSize(
            h,
            COORD {
                x: width,
                y: height,
            },
        );
        let rect = SMALL_RECT {
            left: 0,
            top: 0,
            right: width - 1,
            bottom: height - 1,
        };
        SetConsoleWindowInfo(h, 1, &rect);
    }
}

/// ShellExecuteW 打开路径/网址
pub fn shell_open(target: &str) -> bool {
    let op = to_wide("open");
    let file = to_wide(target);
    let r = unsafe {
        ShellExecuteW(
            0,
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        )
    };
    r as isize > 32
}

/// 以隐藏窗口方式执行命令（cmd /C）
pub fn run_hidden(command: &str) -> bool {
    let mut cmd = to_wide(&format!("cmd.exe /C {}", command));
    let mut si = STARTUPINFOW::default();
    si.dw_flags = STARTF_USESHOWWINDOW;
    si.w_show_window = SW_HIDE as u16;
    let mut pi = PROCESS_INFORMATION::default();
    let ok = unsafe {
        CreateProcessW(
            std::ptr::null(),
            cmd.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            CREATE_NO_WINDOW,
            std::ptr::null_mut(),
            std::ptr::null(),
            &mut si,
            &mut pi,
        )
    };
    if ok != 0 {
        unsafe {
            CloseHandle(pi.h_process);
            CloseHandle(pi.h_thread);
        }
        true
    } else {
        false
    }
}

extern "system" {
    fn SetConsoleCtrlHandler(handler: Option<extern "system" fn(u32) -> BOOL>, add: BOOL) -> BOOL;
}

/// 控制台关闭/ Ctrl+C 回调：发送退出帧
extern "system" fn ctrl_handler(ctrl_type: u32) -> BOOL {
    crate::app::send_exit_frame_from_handler();
    // Ctrl+C / Ctrl+Break 交回默认处理，确保进程真正退出（避免残留实例占用 HID 句柄）；
    // 关窗/注销/关机由系统在回调返回后强制终止
    match ctrl_type {
        0 | 1 => 0,
        _ => 1,
    }
}

/// 安装控制台退出回调，保证关闭窗口前向设备发送结束信号
pub fn install_ctrl_handler() {
    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), 1);
    }
}

/// 隐藏当前控制台窗口（-serve 使用）
pub fn detach_console() {
    unsafe {
        FreeConsole();
    }
}

/// 隐藏控制台窗口但保留进程
pub fn hide_console_window() {
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd != 0 {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}
