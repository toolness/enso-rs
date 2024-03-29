use direct2d::factory::Factory;
use direct2d::render_target::{DxgiSurfaceRenderTarget, RenderTarget};
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::dxgi::{IDXGISurface, IID_IDXGISurface};
use winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::shared::windef::{HDC, HWND, POINT, SIZE};
use winapi::um::d2d1::{
    ID2D1GdiInteropRenderTarget, ID2D1RenderTarget, D2D1_DC_INITIALIZE_MODE_COPY,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
    D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
};
use winapi::um::d3d11::{
    D3D11CreateDevice, ID3D11Device, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_RESOURCE_MISC_GDI_COMPATIBLE, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
};
use winapi::um::d3dcommon::D3D_DRIVER_TYPE_HARDWARE;
use winapi::um::dcommon::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT};
use winapi::um::wingdi::{AC_SRC_ALPHA, BLENDFUNCTION};
use winapi::um::winuser::{UpdateLayeredWindowIndirect, ULW_ALPHA, UPDATELAYEREDWINDOWINFO};
use winapi::Interface;

use super::error::Error;

pub struct Direct3DDevice {
    device: *mut ID3D11Device,
}

impl Direct3DDevice {
    pub fn new() -> Result<Self, Error> {
        let mut device: *mut ID3D11Device = null_mut();

        Error::validate_hresult(unsafe {
            D3D11CreateDevice(
                null_mut(),
                D3D_DRIVER_TYPE_HARDWARE,
                null_mut(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                null_mut(),
                0,
                D3D11_SDK_VERSION,
                &mut device,
                null_mut(),
                null_mut(), // TODO: Consider supplying a pointer to a ID3D11DeviceContext.
            )
        })?;

        Ok(Direct3DDevice { device })
    }

    pub fn get_feature_level(&self) -> u32 {
        unsafe { (*self.device).GetFeatureLevel() }
    }

    pub fn create_texture_2d(&mut self, width: u32, height: u32) -> Result<Direct3DTexture, Error> {
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
    surface: *mut IDXGISurface,
}

impl Direct3DTexture {
    pub fn new(device: *mut ID3D11Device, width: u32, height: u32) -> Result<Self, Error> {
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
                Quality: 0,
            },
            MiscFlags: D3D11_RESOURCE_MISC_GDI_COMPATIBLE,
            CPUAccessFlags: 0,
            Usage: D3D11_USAGE_DEFAULT,
        };
        Error::validate_hresult(unsafe {
            (*device).CreateTexture2D(&desc, null_mut(), &mut texture)
        })?;
        Error::validate_hresult(unsafe {
            (*texture).QueryInterface(&IID_IDXGISurface, &mut surface_ptr)
        })?;

        Ok(Direct3DTexture {
            texture,
            surface: surface_ptr as *mut IDXGISurface,
        })
    }

    pub fn create_d2d_layered_window_renderer(
        &mut self,
    ) -> Result<Direct2DLayeredWindowRenderer, Error> {
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
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
            },
        };
        let mut target: *mut ID2D1RenderTarget = null_mut();
        unsafe {
            let raw_factory = factory.get_raw();
            Error::validate_hresult((*raw_factory).CreateDxgiSurfaceRenderTarget(
                self.surface,
                &props,
                &mut target,
            ))?;

            // I *think* we're passing ownership of this target to the
            // DxgiSurfaceRenderTarget, so it will be responsible for
            // calling Release when it's disposed of.
            let dxgi_target = DxgiSurfaceRenderTarget::from_raw(target);

            let mut gdi_interop_ptr: *mut c_void = null_mut();
            Error::validate_hresult(
                (*target)
                    .QueryInterface(&ID2D1GdiInteropRenderTarget::uuidof(), &mut gdi_interop_ptr),
            )?;

            Ok(Direct2DLayeredWindowRenderer {
                dxgi_target,
                gdi_interop_target: gdi_interop_ptr as *mut ID2D1GdiInteropRenderTarget,
            })
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

pub struct LayeredWindowUpdateOptions {
    pub hwnd: HWND,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct Direct2DLayeredWindowRenderer {
    dxgi_target: DxgiSurfaceRenderTarget,
    gdi_interop_target: *mut ID2D1GdiInteropRenderTarget,
}

impl Direct2DLayeredWindowRenderer {
    pub fn draw_and_update_layered_window<F>(
        &mut self,
        update_options: &LayeredWindowUpdateOptions,
        cb: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(&mut DxgiSurfaceRenderTarget) -> Result<(), Error>,
    {
        self.dxgi_target.begin_draw();
        cb(&mut self.dxgi_target)?;
        self.update_layered_window(update_options)?;
        self.dxgi_target.end_draw()?;
        Ok(())
    }

    pub fn update_layered_window(
        &self,
        update_options: &LayeredWindowUpdateOptions,
    ) -> Result<(), Error> {
        let mut hdc: HDC = null_mut();
        Error::validate_hresult(unsafe {
            (*self.gdi_interop_target).GetDC(D2D1_DC_INITIALIZE_MODE_COPY, &mut hdc)
        })?;
        let blendfunc = BLENDFUNCTION {
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA,
            BlendFlags: 0,
            BlendOp: 0,
        };
        let ppt_src = POINT { x: 0, y: 0 };
        let ppt_dst = POINT {
            x: update_options.x,
            y: update_options.y,
        };
        let psize = SIZE {
            cx: update_options.width as i32,
            cy: update_options.height as i32,
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
            prcDirty: null_mut(),
        };
        unsafe {
            if UpdateLayeredWindowIndirect(update_options.hwnd, &mut update_info) == 0 {
                eprintln!("UpdateLayeredWindowIndirect() failed!");
                return Err(Error::from_winapi());
            }
        }
        Error::validate_hresult(unsafe { (*self.gdi_interop_target).ReleaseDC(null_mut()) })?;
        Ok(())
    }
}

impl Drop for Direct2DLayeredWindowRenderer {
    fn drop(&mut self) {
        unsafe {
            (*self.gdi_interop_target).Release();
        }
    }
}
