// 系统托盘：隐藏控制台、最小化进托盘、双击恢复、右键显示/退出
// 图标来自 exe 内嵌资源（资源组 ID 1），失败时回退系统默认图标

use crate::platform::win::*;
use std::ffi::c_void;
use std::mem::size_of;

const WM_TRAY: u32 = WM_APP + 1;
const MENU_SHOW: i32 = 1001;
const MENU_EXIT: i32 = 1002;

const CLSCTX_ALL: u32 = 0x17;
const COINIT_APARTMENTTHREADED: u32 = 0x2;
const CLSID_TASKBAR_LIST: GUID = GUID {
    data1: 0x56FDF344,
    data2: 0xFD6D,
    data3: 0x11D0,
    data4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
};
const IID_ITASKBAR_LIST: GUID = GUID {
    data1: 0x56FDF342,
    data2: 0xFD6D,
    data3: 0x11D0,
    data4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
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
}

type TaskbarReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type TaskbarHrInitFn = unsafe extern "system" fn(*mut c_void) -> i32;
type TaskbarTabFn = unsafe extern "system" fn(*mut c_void, HWND) -> i32;

/// 通过 ITaskbarList 增删任务栏按钮（修复 Win11 隐藏控制台后残留图标）
unsafe fn taskbar_tab(hwnd: HWND, add: bool) {
    if hwnd == 0 {
        return;
    }
    let hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);
    let mut list: *mut c_void = std::ptr::null_mut();
    if CoCreateInstance(
        &CLSID_TASKBAR_LIST,
        std::ptr::null_mut(),
        CLSCTX_ALL,
        &IID_ITASKBAR_LIST,
        &mut list,
    ) >= 0
        && !list.is_null()
    {
        let vt = *(list as *mut *mut *mut c_void);
        let init: TaskbarHrInitFn = std::mem::transmute(*vt.add(3));
        init(list);
        let tab: TaskbarTabFn = std::mem::transmute(*vt.add(if add { 4 } else { 5 }));
        tab(list, hwnd);
        let release: TaskbarReleaseFn = std::mem::transmute(*vt.add(2));
        release(list);
    }
    if hr >= 0 {
        CoUninitialize();
    }
}

/// 隐藏控制台并移除任务栏按钮
fn hide_console() {
    unsafe {
        let console = GetConsoleWindow();
        if console != 0 {
            ShowWindow(console, SW_HIDE);
            taskbar_tab(console, false);
        }
    }
}

/// 恢复控制台并重新加入任务栏
unsafe fn restore_console() {
    let console = GetConsoleWindow();
    if console != 0 {
        taskbar_tab(console, true);
        ShowWindow(console, SW_RESTORE);
        SetForegroundWindow(console);
    }
}

/// 供 Ctrl+C 回调使用：把控制台收进托盘
pub fn hide_main_console() {
    hide_console();
}

/// 关闭控制台窗口后重启自身（新实例隐藏到托盘）
pub fn relaunch_hidden() {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    if let Ok(exe) = std::env::current_exe() {
        let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
        let _ = std::process::Command::new(exe)
            .args(args)
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn();
    }
}

/// 弹出右键菜单
unsafe fn show_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    if menu == 0 {
        return;
    }
    let show = to_wide("显示控制台");
    let exit = to_wide("退出");
    AppendMenuW(menu, 0, MENU_SHOW as usize, show.as_ptr());
    AppendMenuW(menu, 0, MENU_EXIT as usize, exit.as_ptr());

    let mut pt = POINT::default();
    GetCursorPos(&mut pt);
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD,
        pt.x,
        pt.y,
        0,
        hwnd,
        std::ptr::null(),
    );
    DestroyMenu(menu);

    match cmd {
        MENU_SHOW => restore_console(),
        MENU_EXIT => {
            crate::app::send_exit_frame_from_handler();
            std::process::exit(0);
        }
        _ => {}
    }
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    unsafe {
        if msg == WM_TRAY {
            let event = (l as usize) & 0xFFFF;
            if event == WM_LBUTTONDBLCLK as usize {
                restore_console();
            } else if event == WM_RBUTTONUP as usize {
                show_menu(hwnd);
            }
            return 0;
        }
        if msg == WM_TIMER {
            // 点最小化按钮后把控制台收进托盘（同时移除任务栏按钮）
            let console = GetConsoleWindow();
            if console != 0 && IsIconic(console) != 0 {
                hide_console();
            }
            return 0;
        }
        DefWindowProcW(hwnd, msg, w, l)
    }
}

/// 启动托盘线程，并立即隐藏控制台
pub fn setup() {
    std::thread::spawn(|| unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("N4ProTrayWnd");
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
        if hwnd == 0 {
            crate::log_warn!("托盘窗口创建失败");
            return;
        }

        // 资源图标组 ID=1；失败时用系统默认应用图标
        let mut icon = LoadIconW(instance, 1usize as *const u16);
        if icon == 0 {
            icon = LoadIconW(0, 32512usize as *const u16);
        }

        let mut data = NOTIFYICONDATAW {
            cb_size: size_of::<NOTIFYICONDATAW>() as u32,
            hwnd,
            u_id: 1,
            u_flags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            u_callback_message: WM_TRAY,
            h_icon: icon,
            sz_tip: [0; 128],
            dw_state: 0,
            dw_state_mask: 0,
            sz_info: [0; 256],
            u_version: 0,
            sz_info_title: [0; 64],
            dw_info_flags: 0,
            guid_item: GUID::default(),
            h_balloon_icon: 0,
        };
        for (i, ch) in "N4ProCore".encode_utf16().enumerate() {
            data.sz_tip[i] = ch;
        }
        if Shell_NotifyIconW(NIM_ADD, &mut data) == 0 {
            crate::log_warn!("托盘图标添加失败");
        }

        // 400ms 检测一次最小化
        SetTimer(hwnd, 1, 400, 0);

        // 启动即隐藏到托盘
        hide_console();

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        KillTimer(hwnd, 1);
        Shell_NotifyIconW(NIM_DELETE, &mut data);
    });
}
