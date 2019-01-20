use std::ptr::null_mut;
use winapi::um::d3d11::{
    D3D11CreateDevice,
    ID3D11Device,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_SDK_VERSION
};
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
}

impl Drop for Direct3DDevice {
    fn drop(&mut self) {
        unsafe {
            (*self.device).Release();
        }
    }
}
