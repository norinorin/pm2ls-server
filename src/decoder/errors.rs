use audiopus_sys::opus_strerror;
use core::fmt;
use std::error::Error;
use std::ffi::CStr;

#[derive(Debug, Clone)]
pub struct OpusError {
    pub error: i32,
    pub message: String,
}

impl Error for OpusError {}

impl fmt::Display for OpusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Opus error code {}: {:?}.", self.error, self.message)
    }
}

pub fn get_opus_error(error: i32) -> OpusError {
    OpusError {
        error,
        message: unsafe { CStr::from_ptr(opus_strerror(error)) }
            .to_str()
            .unwrap()
            .to_owned(),
    }
}
