use std::ptr::null_mut;
use std::ffi::CStr;
use std::sync::Once;
use winapi::um::{winuser, wingdi};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::{minwindef, windef};
use direct2d::render_target::DxgiSurfaceRenderTarget;

use super::directx::{
    Direct3DDevice,
    Direct2DLayeredWindowRenderer,
    LayeredWindowUpdateOptions
};

static mut WINDOW_CLASS: minwindef::ATOM = 0;
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
    unsafe fn create_window_class() -> minwindef::ATOM {
        INIT_WINDOW_CLASS.call_once(|| {
            let info = winuser::WNDCLASSEXA {
                cbSize: std::mem::size_of::<winuser::WNDCLASSEXA>() as u32,
                style: 0,
                lpfnWndProc: Some(winuser::DefWindowProcA),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleA(null_mut()),
                hIcon: null_mut(),
                hCursor: null_mut(),
                hbrBackground: wingdi::GetStockObject(wingdi::HOLLOW_BRUSH as i32) as windef::HBRUSH,
                lpszMenuName: null_mut(),
                lpszClassName: window_class_name_ptr(),
                hIconSm: null_mut()
            };

            let window_class = winuser::RegisterClassExA(&info);

            if window_class == 0 {
                panic!("RegisterClassExA() failed with error 0x{:x}!", GetLastError());
            }
            WINDOW_CLASS = window_class;
        });
        WINDOW_CLASS
    }

    unsafe fn create_window(x: i32, y: i32, width: u32, height: u32) -> windef::HWND {
        Self::create_window_class();
        let old_fg_window = winuser::GetForegroundWindow();
        let ex_style = winuser::WS_EX_LAYERED |
            winuser::WS_EX_TOOLWINDOW |
            winuser::WS_EX_TOPMOST |
            winuser::WS_EX_TRANSPARENT;
        let window_style = winuser::WS_VISIBLE | winuser::WS_POPUP;
        let window = winuser::CreateWindowExA(
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
        );

        if window == null_mut() {
            panic!("CreateWindowExA() failed!");
        }
        winuser::SetForegroundWindow(old_fg_window);

        window
    }

    pub fn new(d3d_device: &mut Direct3DDevice, x: i32, y: i32, width: u32, height: u32) -> Self {
        let hwnd = unsafe { Self::create_window(x, y, width, height) };

        let mut texture = d3d_device.create_texture_2d(width, height);
        let renderer = texture.create_d2d_layered_window_renderer();

        TransparentWindow { x, y, width, height, hwnd, renderer }
    }

    pub fn draw_and_update<F>(&mut self, cb: F) where F: FnOnce(&mut DxgiSurfaceRenderTarget) {
        let update_info = LayeredWindowUpdateOptions {
            hwnd: self.hwnd,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height
        };
        match self.renderer.draw_and_update_layered_window(
            &update_info,
            cb
        ) {
            Ok(_) => { println!("Drawing successful.") },
            Err(e) => { println!("Error drawing: {:?}", e) }
        };
    }

    pub fn close(self) {
        // Do nothing, our drop implementation will trigger cleanup.
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
    CStr::from_bytes_with_nul(WINDOW_CLASS_NAME).unwrap().as_ptr()
}
