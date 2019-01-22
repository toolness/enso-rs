use std::fmt;
use std::error;
use std::ptr::null_mut;
use winapi::shared::winerror::HRESULT;
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
    Other(Box<dyn std::error::Error>)
}

impl Error {
    pub fn from_winapi() -> Self {
        Error::WindowsAPI(unsafe { GetLastError() })
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
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_tuple = fmt.debug_tuple("Error");
        let mut out = debug_tuple.field(&self);
        match self {
            Error::WindowsAPI(dword) => {
                if let Ok(desc) = Self::get_winapi_error_desc(*dword) {
                    out = out.field(&desc);
                }
            },
            _ => {}
        };
        out.finish()
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
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
