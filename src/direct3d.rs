use std::ptr::null_mut;
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
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
use winapi::um::d3dcommon::{
    D3D_DRIVER_TYPE_HARDWARE,
};
use winapi::shared::winerror::S_OK;

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
    texture: *mut ID3D11Texture2D
}

impl Direct3DTexture {
    pub fn new(device: *mut ID3D11Device, width: u32, height: u32) -> Self {
        let mut texture: *mut ID3D11Texture2D = null_mut();
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
        Direct3DTexture { texture }
    }
}

impl Drop for Direct3DTexture {
    fn drop(&mut self) {
        unsafe {
            (*self.texture).Release();
        }
    }
}
