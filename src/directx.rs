use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::Interface;
use winapi::shared::windef::{
    HWND,
    HDC,
    POINT,
};
use winapi::um::d3d11::{
    D3D11CreateDevice,
    ID3D11Device,
    ID3D11Texture2D,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC,
    D3D11_BIND_RENDER_TARGET,
    D3D11_RESOURCE_MISC_GDI_COMPATIBLE,
    D3D11_USAGE_DEFAULT
};
use winapi::shared::dxgi::{
    IDXGISurface,
    IID_IDXGISurface,
};
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
use winapi::um::d3dcommon::{
    D3D_DRIVER_TYPE_HARDWARE,
};
use winapi::um::dcommon::{
    D2D1_PIXEL_FORMAT,
    D2D1_ALPHA_MODE_PREMULTIPLIED
};
use winapi::shared::winerror::S_OK;
use winapi::um::d2d1::{
    ID2D1RenderTarget,
    ID2D1GdiInteropRenderTarget,
    D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT,
    D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
    D2D1_FEATURE_LEVEL_DEFAULT,
    D2D1_DC_INITIALIZE_MODE_COPY
};
use winapi::um::wingdi::{
    BLENDFUNCTION,
    AC_SRC_ALPHA
};
use winapi::um::winuser::{
    UpdateLayeredWindowIndirect,
    UPDATELAYEREDWINDOWINFO,
    ULW_ALPHA,
    SIZE
};
use direct2d::factory::Factory;
use direct2d::render_target::DxgiSurfaceRenderTarget;

pub struct Direct3DDevice {
    device: *mut ID3D11Device,
}

impl Direct3DDevice {
    pub unsafe fn new() -> Self {
        let mut device: *mut ID3D11Device = null_mut();

        let result = D3D11CreateDevice(
            null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            null_mut(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            null_mut(),
            0,
            D3D11_SDK_VERSION,
            &mut device,
            null_mut(),
            null_mut()   // TODO: Consider supplying a pointer to a ID3D11DeviceContext.
        );
        if result != S_OK {
            panic!("D3D11CreateDevice() returned {}!", result);
        }

        Direct3DDevice { device }
    }

    pub fn get_feature_level(&self) -> u32 {
        unsafe { (*self.device).GetFeatureLevel() }
    }

    pub fn create_texture_2d(&mut self, width: u32, height: u32) -> Direct3DTexture {
        Direct3DTexture::new(self.device, width, height)
    }
}

impl Drop for Direct3DDevice {
    fn drop(&mut self) {
        unsafe {
            (*self.device).Release();
        }
    }
}

pub struct Direct3DTexture {
    texture: *mut ID3D11Texture2D,
    surface: *mut IDXGISurface
}

impl Direct3DTexture {
    pub fn new(device: *mut ID3D11Device, width: u32, height: u32) -> Self {
        let mut texture: *mut ID3D11Texture2D = null_mut();
        let mut surface_ptr: *mut c_void = null_mut();
        let desc = D3D11_TEXTURE2D_DESC {
            ArraySize: 1,
            BindFlags: D3D11_BIND_RENDER_TARGET,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Width: width,
            Height: height,
            MipLevels: 1,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0
            },
            MiscFlags: D3D11_RESOURCE_MISC_GDI_COMPATIBLE,
            CPUAccessFlags: 0,
            Usage: D3D11_USAGE_DEFAULT
        };
        unsafe {
            let result = (*device).CreateTexture2D(
                &desc,
                null_mut(),
                &mut texture
            );
            if result != S_OK {
                panic!("CreateTexture2D() returned {}!", result);
            }
        };
        unsafe {
            let result = (*texture).QueryInterface(
                &IID_IDXGISurface,
                &mut surface_ptr
            );
            if result != S_OK {
                panic!("QueryInterface(IDXGISurface) returned {}!", result);
            }
        }

        Direct3DTexture { texture, surface: surface_ptr as *mut IDXGISurface }
    }

    pub fn create_d2d_render_target(&mut self) -> GdiFriendlyRenderTarget {
        let factory = Factory::new().expect("Creating Direct2D factory failed");
        let (dpi_x, dpi_y) = factory.get_desktop_dpi();
        let props = D2D1_RENDER_TARGET_PROPERTIES {
            _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            dpiX: dpi_x,
            dpiY: dpi_y,

            // This actually differs from what is advised in the
            // article I read, which recommends D2D1_RENDER_TARGET_TYPE_DEFAULT,
            // which surprises me:
            //
            // https://msdn.microsoft.com/en-us/magazine/ee819134.aspx
            //
            // (Actually, the software rendering part of the article recommends
            // this flag while the hardware rendering part does not, and I suspect
            // the latter was a typo.)
            usage: D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,

            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                format: DXGI_FORMAT_B8G8R8A8_UNORM
            }
        };
        let mut target: *mut ID2D1RenderTarget = null_mut();
        unsafe {
            let raw_factory = factory.get_raw();
            let result = (*raw_factory).CreateDxgiSurfaceRenderTarget(
                self.surface,
                &props,
                &mut target
            );
            if result != S_OK {
                panic!("CreateDxgiSurfaceRenderTarget() returned {}!", result);
            }

            // I *think* we're passing ownership of this target to the
            // DxgiSurfaceRenderTarget, so it will be responsible for
            // calling Release when it's disposed of.
            let dxgi_target = DxgiSurfaceRenderTarget::from_raw(target);

            let mut gdi_interop_ptr: *mut c_void = null_mut();
            let gresult = (*target).QueryInterface(
                &ID2D1GdiInteropRenderTarget::uuidof(),
                &mut gdi_interop_ptr
            );
            if gresult != S_OK {
                panic!("QueryInterface(ID2D1GdiInteropRenderTarget) returned {}!", gresult);
            }

            GdiFriendlyRenderTarget {
                dxgi_target,
                gdi_interop_target: gdi_interop_ptr as *mut ID2D1GdiInteropRenderTarget
            }
        }
    }
}

impl Drop for Direct3DTexture {
    fn drop(&mut self) {
        unsafe {
            (*self.texture).Release();
            (*self.surface).Release();
        }
    }
}

pub struct GdiFriendlyRenderTarget {
    pub dxgi_target: DxgiSurfaceRenderTarget,
    gdi_interop_target: *mut ID2D1GdiInteropRenderTarget,
}

impl GdiFriendlyRenderTarget {
    pub fn update_layered_window(&self, hwnd: HWND) {
        let mut hdc: HDC = null_mut();
        unsafe {
            let result = (*self.gdi_interop_target).GetDC(
                D2D1_DC_INITIALIZE_MODE_COPY,
                &mut hdc
            );
            if result != S_OK {
                panic!("GetDC() returned {:x}!", result);
            }
        }
        let blendfunc = BLENDFUNCTION {
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA,
            BlendFlags: 0,
            BlendOp: 0
        };
        // TODO: A bunch of these points/sizes should not be hard-coded.
        let ppt_src = POINT {
            x: 0,
            y: 0
        };
        let ppt_dst = POINT {
            x: 0,
            y: 0
        };
        let psize = SIZE {
            cx: 100,
            cy: 100
        };
        let mut update_info = UPDATELAYEREDWINDOWINFO {
            cbSize: std::mem::size_of::<UPDATELAYEREDWINDOWINFO>() as u32,
            hdcDst: null_mut(),
            pptDst: &ppt_dst,
            psize: &psize,
            hdcSrc: hdc,
            pptSrc: &ppt_src,
            crKey: 0,
            pblend: &blendfunc,
            dwFlags: ULW_ALPHA,
            prcDirty: null_mut()
        };
        unsafe {
            if UpdateLayeredWindowIndirect(hwnd, &mut update_info) == 0 {
                panic!("UpdateLayeredWindowIndirect() failed!");
            }
            let result = (*self.gdi_interop_target).ReleaseDC(null_mut());
            if result != S_OK {
                panic!("ReleaseDC() returned {}!", result);
            }
        }
    }
}

impl Drop for GdiFriendlyRenderTarget {
    fn drop(&mut self) {
        unsafe {
            (*self.gdi_interop_target).Release();
        }
    }
}
