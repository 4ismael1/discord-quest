#![windows_subsystem = "windows"] 

use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, PostQuitMessage, 
    RegisterClassA, ShowWindow, TranslateMessage, SetTimer,
    CW_USEDEFAULT, MSG, SW_SHOWNORMAL, WINDOW_EX_STYLE, 
    WM_CREATE, WM_DESTROY, WM_PAINT, WM_TIMER, WNDCLASSA, WS_CAPTION, 
    WS_SYSMENU, WS_MINIMIZEBOX,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, CreateSolidBrush, FillRect, SelectObject, DeleteObject,
    CreateFontA, SetBkMode, SetTextColor, DrawTextA,
    CreatePen, MoveToEx, LineTo, RoundRect,
    HDC, HFONT, PAINTSTRUCT, TRANSPARENT, DRAW_TEXT_FORMAT, DT_LEFT, DT_SINGLELINE, DT_VCENTER, 
    DT_CENTER, PS_SOLID, FW_BOLD, FW_NORMAL, FW_SEMIBOLD,
    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 
    DEFAULT_PITCH, FF_SWISS, FONT_CHARSET, FONT_OUTPUT_PRECISION, 
    FONT_CLIP_PRECISION, FONT_QUALITY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::core::PCSTR;
use std::ffi::CString;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::SystemTime;

mod tray;
use tray::create_tray_icon;

const WIDTH: i32 = 380;
const HEIGHT: i32 = 260;
const TIMER_ID: usize = 1;

static START_TIME: AtomicU64 = AtomicU64::new(0);
static GAME_TITLE: OnceLock<String> = OnceLock::new();

#[derive(Debug)]
struct Config {
    title: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: "DiscordQuest".to_string(),
        }
    }
}

fn parse_args() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut config = Config::default();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" => {
                if i + 1 < args.len() {
                    config.title = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    
    config
}

fn create_font(_hdc: HDC, size: i32, weight: i32, name: &str) -> HFONT {
    unsafe {
        let font_name = CString::new(name).unwrap_or_else(|_| CString::new("Segoe UI").unwrap());
        CreateFontA(
            size, 0, 0, 0,
            weight,
            0, 0, 0,
            FONT_CHARSET(DEFAULT_CHARSET.0),
            FONT_OUTPUT_PRECISION(OUT_DEFAULT_PRECIS.0),
            FONT_CLIP_PRECISION(CLIP_DEFAULT_PRECIS.0),
            FONT_QUALITY(CLEARTYPE_QUALITY.0),
            (DEFAULT_PITCH.0 as u32) | (FF_SWISS.0 as u32),
            PCSTR(font_name.as_ptr() as *const u8),
        )
    }
}

fn format_elapsed(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

fn draw_rounded_rect(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32, radius: i32, fill_color: COLORREF, border_color: COLORREF) {
    unsafe {
        let brush = CreateSolidBrush(fill_color);
        let pen = CreatePen(PS_SOLID, 1, border_color);
        let old_brush = SelectObject(hdc, brush.into());
        let old_pen = SelectObject(hdc, pen.into());
        let _ = RoundRect(hdc, left, top, right, bottom, radius, radius);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
}

fn draw_text_at(hdc: HDC, text: &str, x: i32, y: i32, w: i32, h: i32, color: COLORREF, font: HFONT, flags: DRAW_TEXT_FORMAT) {
    unsafe {
        let old_font = SelectObject(hdc, font.into());
        SetTextColor(hdc, color);
        SetBkMode(hdc, TRANSPARENT);
        let cstr = CString::new(text).unwrap_or_else(|_| CString::new("?").unwrap());
        let bytes = cstr.as_bytes();
        let mut rc = RECT { left: x, top: y, right: x + w, bottom: y + h };
        DrawTextA(hdc, &mut Vec::from(bytes), &mut rc, flags);
        SelectObject(hdc, old_font);
    }
}

fn paint_window(hwnd: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        
        let mut client_rect = RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);
        let cw = client_rect.right;
        let ch = client_rect.bottom;

        // ── Background ──
        let bg_brush = CreateSolidBrush(COLORREF(0x00161216));
        FillRect(hdc, &client_rect, bg_brush);
        let _ = DeleteObject(bg_brush.into());

        // ── Top accent bar ──
        let accent_rect = RECT { left: 0, top: 0, right: cw, bottom: 3 };
        let accent_brush = CreateSolidBrush(COLORREF(0x00F26558)); // #5865F2 in BGR
        FillRect(hdc, &accent_rect, accent_brush);
        let _ = DeleteObject(accent_brush.into());

        // ── Fonts ──
        let font_title = create_font(hdc, 22, FW_BOLD.0 as i32, "Segoe UI");
        let font_label = create_font(hdc, 13, FW_SEMIBOLD.0 as i32, "Segoe UI");
        let font_value = create_font(hdc, 13, FW_NORMAL.0 as i32, "Segoe UI");
        let font_timer = create_font(hdc, 32, FW_BOLD.0 as i32, "Cascadia Code");
        let font_small = create_font(hdc, 11, FW_NORMAL.0 as i32, "Segoe UI");

        let title = GAME_TITLE.get().map(|s| s.as_str()).unwrap_or("DiscordQuest");
        let dt_left_single = DT_LEFT | DT_SINGLELINE | DT_VCENTER;
        let dt_left = DT_LEFT | DT_SINGLELINE;
        let dt_center = DT_CENTER | DT_SINGLELINE | DT_VCENTER;

        // ── Status pill ──  
        draw_rounded_rect(hdc, 20, 18, 90, 38, 10, 
            COLORREF(0x001A3A1A),
            COLORREF(0x00287F28),
        );
        let dot_color = COLORREF(0x0087F257); // #57F287 green
        draw_text_at(hdc, "* Activo", 26, 18, 60, 20, dot_color, font_label, dt_left_single);

        // ── Game title ──
        draw_text_at(hdc, title, 20, 48, cw - 40, 28, COLORREF(0x00F4F0F0), font_title, dt_left);

        // ── Separator line ──
        let sep_pen = CreatePen(PS_SOLID, 1, COLORREF(0x003A2A2A));
        let old_pen = SelectObject(hdc, sep_pen.into());
        let _ = MoveToEx(hdc, 20, 84, None);
        let _ = LineTo(hdc, cw - 20, 84);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(sep_pen.into());

        // ── Info cards area ──
        let card_y = 96;
        let card_h = 52;
        let half_w = (cw - 52) / 2;
        
        // Card 1: Estado
        draw_rounded_rect(hdc, 20, card_y, 20 + half_w, card_y + card_h, 8,
            COLORREF(0x001E1A1E),
            COLORREF(0x002A262A),
        );
        draw_text_at(hdc, "ESTADO", 30, card_y + 8, half_w - 20, 14, COLORREF(0x007A647A), font_label, dt_left);
        draw_text_at(hdc, "Simulando", 30, card_y + 26, half_w - 20, 16, COLORREF(0x0087F257), font_value, dt_left);

        // Card 2: Tipo
        let card2_x = 20 + half_w + 12;
        draw_rounded_rect(hdc, card2_x, card_y, card2_x + half_w, card_y + card_h, 8,
            COLORREF(0x001E1A1E),
            COLORREF(0x002A262A),
        );
        draw_text_at(hdc, "TIPO", card2_x + 10, card_y + 8, half_w - 20, 14, COLORREF(0x007A647A), font_label, dt_left);
        draw_text_at(hdc, "Quest Runner", card2_x + 10, card_y + 26, half_w - 20, 16, COLORREF(0x00B0A0B0), font_value, dt_left);

        // ── Timer section ──
        let timer_y = card_y + card_h + 16;
        draw_rounded_rect(hdc, 20, timer_y, cw - 20, timer_y + 58, 10,
            COLORREF(0x00201820), 
            COLORREF(0x002A262A),
        );
        draw_text_at(hdc, "TIEMPO ACTIVO", 30, timer_y + 6, 120, 14, COLORREF(0x007A647A), font_label, dt_left);
        
        let start = START_TIME.load(Ordering::Relaxed);
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();
        let elapsed = if now > start { now - start } else { 0 };
        let time_str = format_elapsed(elapsed);
        draw_text_at(hdc, &time_str, 20, timer_y + 20, cw - 40, 38, COLORREF(0x00F4F0F0), font_timer, dt_center);

        // ── Footer ──
        let footer_y = ch - 24;
        let footer_sep = RECT { left: 0, top: footer_y - 1, right: cw, bottom: footer_y };
        let footer_line = CreateSolidBrush(COLORREF(0x002A222A));
        FillRect(hdc, &footer_sep, footer_line);
        let _ = DeleteObject(footer_line.into());
        
        draw_text_at(hdc, "DiscordQuest - No cerrar esta ventana", 20, footer_y + 2, cw - 40, 16, COLORREF(0x00504050), font_small, dt_left);

        // ── Cleanup fonts ──
        let _ = DeleteObject(font_title.into());
        let _ = DeleteObject(font_label.into());
        let _ = DeleteObject(font_value.into());
        let _ = DeleteObject(font_timer.into());
        let _ = DeleteObject(font_small.into());

        let _ = EndPaint(hwnd, &ps);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Set up 1-second timer for elapsed time updates
            let _ = SetTimer(Some(hwnd), TIMER_ID, 1000, None);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                // Repaint to update the timer
                let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            paint_window(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

fn create_native_window(title: &str) -> Result<(HWND, HINSTANCE), Box<dyn std::error::Error>> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        let class_name = CString::new("DiscordQuestWindow")?;
        let window_title = CString::new(format!("DiscordQuest - {}", title))?;

        // Dark background brush
        let brush = CreateSolidBrush(COLORREF(0x00161216));

        let wc = WNDCLASSA {
            lpfnWndProc: Some(window_proc),
            hInstance: HINSTANCE(instance.0),
            lpszClassName: PCSTR(class_name.as_ptr() as *const u8),
            hbrBackground: brush,
            ..Default::default()
        };

        RegisterClassA(&wc);

        // Fixed window (not resizable) with caption and minimize
        let style = WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;

        let hwnd = CreateWindowExA(
            WINDOW_EX_STYLE(0),
            PCSTR(class_name.as_ptr() as *const u8),
            PCSTR(window_title.as_ptr() as *const u8),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WIDTH,
            HEIGHT,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        ); 
        match hwnd {
            Ok(hwnd) if !hwnd.0.is_null() => Ok((hwnd, HINSTANCE(instance.0))),
            _ => Err("Error al crear ventana".into()),
        }
    }
}

fn main() {
    let config = parse_args();

    // Store start time
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();
    START_TIME.store(now, Ordering::Relaxed);

    // Store title globally for paint
    GAME_TITLE.set(config.title.clone()).ok();
    
    let tray_menu = tray_icon::menu::Menu::new();
    let quit_i = tray_icon::menu::MenuItem::new("Cerrar", true, None);
    let show_i = tray_icon::menu::MenuItem::new("Mostrar", true, None);

    let _tray_menu = tray_menu.append_items(&[
        &show_i,
        &tray_icon::menu::PredefinedMenuItem::separator(),
        &quit_i
    ]);

    let _tray = create_tray_icon(tray_menu, &format!("DiscordQuest - {}", &config.title));

    let (hwnd, _instance) = match create_native_window(&config.title) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error al crear ventana: {}", e);
            return;
        }
    };
    
    unsafe { 
        let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        
        let mut msg = MSG::default();
        loop {
            // Handle tray events
            if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                if event.id == quit_i.id() {
                    PostQuitMessage(0);
                }

                if event.id == show_i.id() {
                    let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
                    let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
                }
            }

            let ret = GetMessageA(&mut msg, None, 0, 0);
            if ret.0 == 0 || ret.0 == -1 {
                break;
            }
            
            let _ = TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}