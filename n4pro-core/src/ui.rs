// 日志窗口 + 系统托盘（GUI 子系统，无控制台窗口）
// 关闭/最小化窗口只隐藏；托盘右键“退出”才真正结束并发送 CLE+DC

use crate::platform::win::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Mutex, OnceLock};

const WM_TRAY: u32 = WM_APP + 1;
const MENU_SHOW: i32 = 1001;
const MENU_EXIT: i32 = 1002;

const WM_SIZE: u32 = 0x0005;
const WM_CLOSE: u32 = 0x0010;
const WM_QUERYENDSESSION: u32 = 0x0011;
const WM_DESTROY: u32 = 0x0002;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_SETFONT: u32 = 0x0030;
const EM_SETSEL: u32 = 0x00B1;
const EM_SCROLLCARET: u32 = 0x00B7;
const SIZE_MINIMIZED: usize = 1;

const WS_CHILD: u32 = 0x4000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_VSCROLL: u32 = 0x0020_0000;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_READONLY: u32 = 0x0800;
const SW_SHOW: i32 = 5;

const MAX_LINES: usize = 50;

static EDIT_HWND: AtomicIsize = AtomicIsize::new(0);
static BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static RECEIVER: OnceLock<Mutex<Receiver<String>>> = OnceLock::new();
static LINES: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

#[link(name = "user32")]
extern "system" {
    fn SetWindowTextW(hwnd: HWND, text: *const u16) -> BOOL;
    fn SendMessageW(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize;
    fn MoveWindow(hwnd: HWND, x: i32, y: i32, w: i32, h: i32, repaint: BOOL) -> BOOL;
    fn GetClientRect(hwnd: HWND, rect: *mut RECT) -> BOOL;
    fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn PostQuitMessage(code: i32);
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: COLORREF) -> HANDLE;
}

#[link(name = "kernel32")]
extern "system" {
    fn AttachConsole(pid: u32) -> BOOL;
    fn AllocConsole() -> BOOL;
}

/// --console 调试模式：优先附加父控制台，失败则新建控制台
pub fn attach_or_alloc_console() {
    unsafe {
        if AttachConsole(0xFFFF_FFFF) == 0 {
            AllocConsole();
        }
    }
}

fn lines() -> &'static Mutex<VecDeque<String>> {
    LINES.get_or_init(|| Mutex::new(VecDeque::new()))
}

unsafe fn refresh_edit() {
    let edit = EDIT_HWND.load(Ordering::Relaxed);
    if edit == 0 {
        return;
    }
    let text = {
        let guard = lines().lock().unwrap_or_else(|e| e.into_inner());
        let mut s = String::new();
        for line in guard.iter() {
            s.push_str(line);
        }
        s
    };
    let wide = to_wide(&text);
    SetWindowTextW(edit, wide.as_ptr());
    let end = wide.len().saturating_sub(1) as isize;
    SendMessageW(edit, EM_SETSEL, end.max(0) as usize, end);
    SendMessageW(edit, EM_SCROLLCARET, 0, 0);
}

unsafe fn pump_logs() {
    let Some(receiver) = RECEIVER.get() else {
        return;
    };
    let Ok(rx) = receiver.lock() else {
        return;
    };
    let mut changed = false;
    loop {
        match rx.try_recv() {
            Ok(line) => {
                let mut guard = lines().lock().unwrap_or_else(|e| e.into_inner());
                guard.push_back(line);
                while guard.len() > MAX_LINES {
                    guard.pop_front();
                }
                changed = true;
            }
            Err(_) => break,
        }
    }
    drop(rx);
    if changed {
        refresh_edit();
    }
}

unsafe fn show_window(hwnd: HWND) {
    ShowWindow(hwnd, SW_SHOW);
    ShowWindow(hwnd, SW_RESTORE);
    SetForegroundWindow(hwnd);
}

unsafe fn layout(hwnd: HWND) {
    let mut rect = RECT::default();
    if GetClientRect(hwnd, &mut rect) == 0 {
        return;
    }
    let edit = EDIT_HWND.load(Ordering::Relaxed);
    if edit != 0 {
        MoveWindow(edit, 0, 0, rect.right.max(1), rect.bottom.max(1), 1);
    }
}

unsafe fn show_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    if menu == 0 {
        return;
    }
    let show = to_wide("显示日志");
    let quit = to_wide("退出");
    AppendMenuW(menu, 0, MENU_SHOW as usize, show.as_ptr());
    AppendMenuW(menu, 0, MENU_EXIT as usize, quit.as_ptr());

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
        MENU_SHOW => show_window(hwnd),
        MENU_EXIT => {
            crate::app::send_exit_frame_from_handler();
            std::process::exit(0);
        }
        _ => {}
    }
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    unsafe {
        match msg {
            WM_TRAY => {
                let event = (l as usize) & 0xFFFF;
                if event == WM_LBUTTONDBLCLK as usize {
                    show_window(hwnd);
                } else if event == WM_RBUTTONUP as usize {
                    show_menu(hwnd);
                }
                0
            }
            WM_SIZE => {
                if w == SIZE_MINIMIZED {
                    ShowWindow(hwnd, SW_HIDE);
                } else {
                    layout(hwnd);
                }
                0
            }
            WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                0
            }
            WM_QUERYENDSESSION => {
                crate::app::send_exit_frame_from_handler();
                1
            }
            WM_TIMER => {
                pump_logs();
                0
            }
            WM_CTLCOLOREDIT => {
                let hdc = w as HDC;
                SetTextColor(hdc, 0x00D4D4D4);
                SetBkColor(hdc, 0x001E1E1E);
                BG_BRUSH.load(Ordering::Relaxed)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}

unsafe fn run_window(show_initial: bool) {
    let instance = GetModuleHandleW(std::ptr::null());
    let class_name = to_wide("N4ProLogWnd");
    let brush = CreateSolidBrush(0x001E1E1E);
    BG_BRUSH.store(brush, Ordering::Relaxed);

    let mut icon = LoadIconW(instance, 1usize as *const u16);
    if icon == 0 {
        icon = LoadIconW(0, 32512usize as *const u16);
    }

    let wc = WNDCLASSW {
        style: 0,
        lpfn_wnd_proc: Some(wnd_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: instance,
        h_icon: icon,
        h_cursor: 0,
        hbr_background: brush,
        lpsz_menu_name: std::ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    let title = to_wide("N4ProCore - N4 Pro 日志");
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_OVERLAPPEDWINDOW,
        120,
        120,
        920,
        600,
        0,
        0,
        instance,
        std::ptr::null_mut(),
    );
    if hwnd == 0 {
        return;
    }

    let edit_class = to_wide("EDIT");
    let empty = to_wide("");
    let edit = CreateWindowExW(
        0,
        edit_class.as_ptr(),
        empty.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
        0,
        0,
        900,
        560,
        hwnd,
        0,
        instance,
        std::ptr::null_mut(),
    );
    EDIT_HWND.store(edit, Ordering::Relaxed);

    let face = to_wide("Consolas");
    let font = CreateFontW(-16, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 4, 0, face.as_ptr());
    SendMessageW(edit, WM_SETFONT, font as usize, 1);
    refresh_edit();
    layout(hwnd);

    let mut data = NOTIFYICONDATAW {
        cb_size: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
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
    Shell_NotifyIconW(NIM_ADD, &mut data);

    SetTimer(hwnd, 1, 100, 0);
    if show_initial {
        show_window(hwnd);
    }

    let mut msg = MSG::default();
    while GetMessageW(&mut msg, 0, 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    KillTimer(hwnd, 1);
    Shell_NotifyIconW(NIM_DELETE, &mut data);
}

/// 启动日志窗口与托盘；show_initial=false 时仅托盘可见
pub fn start(show_initial: bool) {
    let (tx, rx) = sync_channel::<String>(256);
    // 先装载启动阶段的历史日志，再注册发送端，避免重复
    if let Ok(mut guard) = lines().lock() {
        for line in crate::logging::history_snapshot() {
            guard.push_back(line);
        }
        while guard.len() > MAX_LINES {
            guard.pop_front();
        }
    }
    crate::logging::set_ui_sender(tx);
    let _ = RECEIVER.set(Mutex::new(rx));
    std::thread::spawn(move || unsafe { run_window(show_initial) });
}
