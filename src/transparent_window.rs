use std::ptr::null_mut;
use std::ffi::CStr;
use std::sync::Once;
use winapi::um::{winuser, wingdi};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::{minwindef, windef};
use direct2d::render_target::DxgiSurfaceRenderTarget;

use super::directx::{
    Direct3DDevice,
    Direct2DLayeredWindowRenderer,
    LayeredWindowUpdateOptions
};
use super::error::Error;

static mut WINDOW_CLASS: Result<minwindef::ATOM, minwindef::DWORD> = Ok(0);
static INIT_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &'static [u8] = b"EnsoTransparentWindow\0";

pub struct TransparentWindow {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    hwnd: windef::HWND,
    renderer: Direct2DLayeredWindowRenderer
}

impl TransparentWindow {
    fn create_window_class() -> Result<minwindef::ATOM, Error> {
        INIT_WINDOW_CLASS.call_once(|| {
            let bg = unsafe { wingdi::GetStockObject(wingdi::HOLLOW_BRUSH as i32) as windef::HBRUSH };
            let info = winuser::WNDCLASSEXA {
                cbSize: std::mem::size_of::<winuser::WNDCLASSEXA>() as u32,
                style: 0,
                lpfnWndProc: Some(winuser::DefWindowProcA),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: unsafe { GetModuleHandleA(null_mut()) },
                hIcon: null_mut(),
                hCursor: null_mut(),
                hbrBackground: bg,
                lpszMenuName: null_mut(),
                lpszClassName: unsafe { window_class_name_ptr() },
                hIconSm: null_mut()
            };

            let window_class = unsafe { winuser::RegisterClassExA(&info) };

            unsafe {
                WINDOW_CLASS = if window_class == 0 {
                    Err(Error::get_last_windows_api_error())
                } else {
                    Ok(window_class)
                };
            }
        });
        let result = unsafe { WINDOW_CLASS };
        match result {
            Ok(atom) => Ok(atom),
            Err(code) => Err(Error::WindowsAPI(code))
        }
    }

    fn create_window(x: i32, y: i32, width: u32, height: u32) -> Result<windef::HWND, Error> {
        Self::create_window_class()?;
        let old_fg_window = unsafe { winuser::GetForegroundWindow() };
        let ex_style = winuser::WS_EX_LAYERED |
            winuser::WS_EX_TOOLWINDOW |
            winuser::WS_EX_TOPMOST |
            winuser::WS_EX_TRANSPARENT;
        let window_style = winuser::WS_VISIBLE | winuser::WS_POPUP;
        let window = unsafe { winuser::CreateWindowExA(
            ex_style,                       /* dwExStyle    */
            window_class_name_ptr(),        /* lpClassName  */
            null_mut(),                     /* lpWindowName */
            window_style,                   /* dwStyle      */
            x,                              /* x            */
            y,                              /* y            */
            width as i32,                   /* nWidth       */
            height as i32,                  /* nHeight      */
            null_mut(),                     /* hWndParent   */
            null_mut(),                     /* hMenu        */
            GetModuleHandleA(null_mut()),   /* hInstance    */
            null_mut()                      /* lpParam      */
        ) };

        if window == null_mut() {
            return Err(Error::from_winapi());
        }
        unsafe { winuser::SetForegroundWindow(old_fg_window) };

        Ok(window)
    }

    pub fn new(d3d_device: &mut Direct3DDevice, x: i32, y: i32, width: u32, height: u32) -> Result<Self, Error> {
        let hwnd = Self::create_window(x, y, width, height)?;

        let mut texture = d3d_device.create_texture_2d(width, height)?;
        let renderer = texture.create_d2d_layered_window_renderer()?;

        Ok(TransparentWindow { x, y, width, height, hwnd, renderer })
    }

    pub fn draw_and_update<F>(&mut self, cb: F) -> Result<(), Error> where F: FnOnce(&mut DxgiSurfaceRenderTarget) -> Result<(), Error> {
        let update_info = LayeredWindowUpdateOptions {
            hwnd: self.hwnd,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height
        };
        self.renderer.draw_and_update_layered_window(
            &update_info,
            cb
        )
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for TransparentWindow {
    fn drop(&mut self) {
        println!("Destroying transparent window.");
        if unsafe { winuser::DestroyWindow(self.hwnd) } == 0 {
            println!("Warning: Couldn't destroy transparent window!");
        }
    }
}

unsafe fn window_class_name_ptr() -> *const i8 {
    // We're safe unwrapping this because an error will only
    // occur if WINDOW_CLASS_NAME isn't nul-terminated or
    // contains interior nul bytes, which we know won't
    // be the case at runtime.
    CStr::from_bytes_with_nul(WINDOW_CLASS_NAME).unwrap().as_ptr()
}

#[test]
fn test_window_class_name_ptr() {
    unsafe { window_class_name_ptr(); }
}
