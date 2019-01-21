use std::ptr::null_mut;
use std::ffi::CStr;
use std::sync::Once;
use winapi::um::{winuser, wingdi};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::{minwindef, windef};

static mut WINDOW_CLASS: minwindef::ATOM = 0;
static INIT_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &'static [u8] = b"EnsoTransparentWindow\0";

pub struct TransparentWindow {
    hwnd: windef::HWND
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

    unsafe fn create_window(width: u32, height: u32) -> windef::HWND {
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
            0,                              /* x            */
            0,                              /* y            */
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

        use super::directx::Direct3DDevice;
        use direct2d::render_target::RenderTarget;

        // Create a texture and bind Direct2D to it,
        // as per https://msdn.microsoft.com/en-us/magazine/ee819134.aspx.

        let mut d3d = Direct3DDevice::new();
        println!("Created Direct3D device with feature level 0x{:x}.", d3d.get_feature_level());
        let mut texture = d3d.create_texture_2d(width, height);
        println!("Created 2D texture.");
        let mut target = texture.create_d2d_render_target();
        println!("Created Direct2D render target.");

        target.dxgi_target.begin_draw();
        target.dxgi_target.clear(0xFF_FF_FF);
        target.update_layered_window(window);
        println!("Updated layered window.");
        match target.dxgi_target.end_draw() {
            Ok(_) => { println!("Drawing successful.") },
            Err(e) => { println!("Error drawing: {:?}", e) }
        };
        window
    }

    pub fn new(width: u32, height: u32) -> Self {
        TransparentWindow {
            hwnd: unsafe { Self::create_window(width, height) }
        }
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
