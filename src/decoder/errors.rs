use core::fmt;

#[derive(Debug, Clone)]
pub struct OpusError {
    pub error: i32,
}

impl fmt::Display for OpusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Opus error code {}: {}.",
            self.error,
            match self.error {
                -1 => "bad argument",
                -2 => "buffer too small",
                -3 => "internal error",
                -4 => "invalid packet",
                -5 => "unimplemented",
                -6 => "invalid state",
                -7 => "allocation failed",
                _ => "unknown error.",
            }
        )
    }
}
