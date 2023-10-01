#![windows_subsystem = "windows"]

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::{
        Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT},
        System::LibraryLoader::GetModuleHandleA,
    },
    Win32::{
        Graphics::Gdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC,
            DeleteObject, FillRect, InvalidateRect, SelectObject, TextOutW, SRCCOPY,
        },
        UI::WindowsAndMessaging::*,
    },
};

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = s!("window");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("curvel"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            300,
            150,
            None,
            None,
            instance,
            None,
        );

        let mut message = MSG::default();

        while GetMessageA(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }

        Ok(())
    }
}

static mut OLD: POINT = POINT { x: 0, y: 0 };
static mut NEW: POINT = POINT { x: 0, y: 0 };
extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                let _ = GetCursorPos(&mut NEW);
                OLD = NEW;
                SetTimer(window, 1, 100, None);
                LRESULT(0)
            }
            WM_TIMER => {
                OLD = NEW;
                let _ = GetCursorPos(&mut NEW);
                InvalidateRect(window, None, true);
                LRESULT(0)
            }
            WM_PAINT => {
                let mut rect = RECT::default();
                let _ = GetClientRect(window, &mut rect);

                let mut ps: PAINTSTRUCT = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut ps);

                let hdc_mem = CreateCompatibleDC(hdc);
                let hbm_mem = CreateCompatibleBitmap(hdc, rect.right, rect.bottom);
                SelectObject(hdc_mem, hbm_mem);

                let h_white_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
                FillRect(hdc_mem, &rect, h_white_brush);

                let vel: i32 =
                    (((NEW.x - OLD.x).pow(2) + (NEW.y - OLD.y).pow(2)) as f64).sqrt() as i32;
                TextOutW(
                    hdc_mem,
                    10,
                    10,
                    HSTRING::from(vel.to_string() + "px/100ms").as_wide(),
                );

                let _ = BitBlt(hdc, 0, 0, rect.right, rect.bottom, hdc_mem, 0, 0, SRCCOPY);

                DeleteObject(hbm_mem);
                DeleteObject(h_white_brush);
                DeleteDC(hdc_mem);

                EndPaint(window, &ps);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
