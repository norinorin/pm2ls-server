use audiopus_sys::opus_strerror;
use core::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct OpusError {
    pub error: i32,
}

impl Error for OpusError {}

impl fmt::Display for OpusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Opus error code {}: {:?}.", self.error, unsafe {
            opus_strerror(self.error)
        })
    }
}
