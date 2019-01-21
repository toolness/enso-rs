use std::fmt;
use std::error;
use winapi::shared::winerror::HRESULT;

#[derive(Debug)]
pub enum Error {
    WindowsCOM(HRESULT),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("Error")
            .field(&self)
            .finish()
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
