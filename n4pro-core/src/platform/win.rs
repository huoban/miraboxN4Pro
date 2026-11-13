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
#[derive(Clone, Copy)]
pub union INPUT_U {
    pub ki: KEYBDINPUT,
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

/// 发送一次键盘按键（含修饰键），flags 使用 KEYEVENTF_*
pub fn key_press(vk: u16, ctrl: bool, shift: bool, alt: bool, win: bool) {
    unsafe {
        if ctrl {
            keybd_event(0x11, 0, 0, 0);
        }
        if shift {
            keybd_event(0x10, 0, 0, 0);
        }
        if alt {
            keybd_event(0x12, 0, 0, 0);
        }
        if win {
            keybd_event(0x5B, 0, 0, 0);
        }
        // 扩展键（方向键/媒体键等）需要 EXTENDEDKEY
        let extended = matches!(vk, 0x25..=0x28 | 0x21..=0x23 | 0x24 | 0x2D..=0x2E | 0xA6..=0xB7);
        let flags = if extended { KEYEVENTF_EXTENDEDKEY } else { 0 };
        keybd_event(vk as u8, 0, flags, 0);
        keybd_event(vk as u8, 0, flags | KEYEVENTF_KEYUP, 0);
        if win {
            keybd_event(0x5B, 0, KEYEVENTF_KEYUP, 0);
        }
        if alt {
            keybd_event(0x12, 0, KEYEVENTF_KEYUP, 0);
        }
        if shift {
            keybd_event(0x10, 0, KEYEVENTF_KEYUP, 0);
        }
        if ctrl {
            keybd_event(0x11, 0, KEYEVENTF_KEYUP, 0);
        }
    }
}

/// 直接发送媒体/音量键（按下+抬起）
pub fn media_key(vk: u16) {
    unsafe {
        keybd_event(vk as u8, 0, KEYEVENTF_EXTENDEDKEY, 0);
        keybd_event(vk as u8, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
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
extern "system" fn ctrl_handler(_ctrl_type: u32) -> BOOL {
    crate::app::send_exit_frame_from_handler();
    1
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
