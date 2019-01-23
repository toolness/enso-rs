use std::fmt;
use std::error;
use std::ptr::null_mut;
use winapi::shared::winerror::{HRESULT, S_OK};
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::{
    FormatMessageA,
    FORMAT_MESSAGE_FROM_SYSTEM
};

#[derive(Debug)]
pub enum Error {
    WindowsCOM(HRESULT),
    WindowsAPI(DWORD),
    Direct2DWithRenderTag(direct2d::error::Error, &'static str),
    Other(Box<dyn std::error::Error>)
}

impl Error {
    pub fn from_winapi() -> Self {
        Error::WindowsAPI(unsafe { GetLastError() })
    }

    pub fn validate_hresult(hresult: HRESULT) -> Result<(), Error> {
        if hresult == S_OK {
            Ok(())
        } else {
            Err(Error::WindowsCOM(hresult))
        }
    }

    fn get_winapi_error_desc(dword: DWORD) -> Result<String, Error> {
        let mut buf: [u8; 1024] = [0; 1024];
        let result = unsafe {
            FormatMessageA(
                FORMAT_MESSAGE_FROM_SYSTEM,
                null_mut(),
                dword,
                0,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as u32,
                null_mut()
            )
        };
        if result == 0 {
            return Err(Error::from_winapi());
        }
        let strlen = result as usize;
        let s = std::str::from_utf8(&buf[0..strlen])?;
        Ok(s.into())
    }

    fn include_winapi_error_desc(out: &mut fmt::DebugTuple<'_, '_>, dword: DWORD) {
        if let Ok(desc) = Self::get_winapi_error_desc(dword) {
            out.field(&desc);
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut out = fmt.debug_tuple("Error");
        out.field(&self);
        match self {
            Error::WindowsAPI(dword) => Self::include_winapi_error_desc(&mut out, *dword),
            // Apparently FormatMessage can also deal with HRESULTs too...
            Error::WindowsCOM(hresult) => Self::include_winapi_error_desc(&mut out, *hresult as u32),
            _ => {}
        };
        out.finish()
    }
}

impl error::Error for Error {
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error::Other(Box::new(err))
    }
}

#[test]
fn test_from_winapi_works() {
    use winapi::um::winuser::GetClientRect;
    use winapi::shared::winerror::ERROR_INVALID_WINDOW_HANDLE;

    let result = unsafe { GetClientRect(null_mut(), null_mut()) };
    assert_eq!(result, 0);
    let err = Error::from_winapi();
    match err {
        Error::WindowsAPI(dword) => {
            assert_eq!(dword, ERROR_INVALID_WINDOW_HANDLE);
        },
        _ => panic!()
    }
    assert_eq!(err.to_string(), "Error(WindowsAPI(1400), \"Invalid window handle.\\r\\n\")");
}

#[test]
fn test_from_winapi_works_with_invalid_dword() {
    // Bit 29 is an application-defined error code, so Windows won't be
    // able to find a valid string for it:
    //
    // https://msdn.microsoft.com/en-us/ms680627
    let err = Error::WindowsAPI(1 << 29);
    assert_eq!(err.to_string(), "Error(WindowsAPI(536870912))");
}