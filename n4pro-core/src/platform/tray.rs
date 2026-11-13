// 系统托盘：隐藏控制台、最小化进托盘、双击恢复、右键显示/退出
// 图标来自 exe 内嵌资源（资源组 ID 1），失败时回退系统默认图标

use crate::platform::win::*;
use std::mem::size_of;

const WM_TRAY: u32 = WM_APP + 1;
const MENU_SHOW: i32 = 1001;
const MENU_EXIT: i32 = 1002;

/// 恢复控制台窗口
unsafe fn restore_console() {
    let console = GetConsoleWindow();
    if console != 0 {
        ShowWindow(console, SW_RESTORE);
        SetForegroundWindow(console);
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
            // 点最小化按钮后把控制台收进托盘
            let console = GetConsoleWindow();
            if console != 0 && IsIconic(console) != 0 {
                ShowWindow(console, SW_HIDE);
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
        let console = GetConsoleWindow();
        if console != 0 {
            ShowWindow(console, SW_HIDE);
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        KillTimer(hwnd, 1);
        Shell_NotifyIconW(NIM_DELETE, &mut data);
    });
}
