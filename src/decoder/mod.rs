extern crate audiopus_sys;

use audiopus_sys::{opus_decode, opus_decode_float, opus_decoder_create, OPUS_OK};
use audiopus_sys::{opus_decoder_destroy, OpusDecoder as OpusDecoderState};

mod errors;

use errors::OpusError;

pub struct OpusDecoder {
    state: *mut OpusDecoderState,
    sample_rate: i32,
    channels: i32,
}

impl OpusDecoder {
    pub fn new(sample_rate: i32, channels: i32) -> Result<Self, OpusError> {
        let mut error: i32 = OPUS_OK;
        let state: *mut OpusDecoderState =
            unsafe { opus_decoder_create(sample_rate, channels, &mut error) };

        if error != 0 {
            return Err(OpusError { error });
        }

        Ok(OpusDecoder {
            state,
            sample_rate,
            channels,
        })
    }

    pub fn destroy(&self) {
        unsafe { opus_decoder_destroy(self.state) }
    }

    pub fn decode_float(&self, encoded: &[u8], fec: bool) -> Result<Vec<f32>, OpusError> {
        let frame_size = 20 * self.sample_rate / 1000 * self.channels;
        let mut decoded =
            vec![0.0; self.channels as usize * frame_size as usize * std::mem::size_of::<f32>()];
        let written = unsafe {
            opus_decode_float(
                self.state,
                encoded.as_ptr(),
                encoded.len().try_into().expect("data is out of range"),
                decoded.as_mut_ptr(),
                20 * self.sample_rate / 1000 * self.channels,
                fec.into(),
            )
        };

        if written < 0 {
            return Err(OpusError { error: written });
        }

        Ok(decoded)
    }

    pub fn decode(&self, encoded: &[u8], fec: bool) -> Result<Vec<i16>, OpusError> {
        let frame_size = 20 * self.sample_rate / 1000 * self.channels;
        let mut decoded =
            vec![0; self.channels as usize * frame_size as usize * std::mem::size_of::<i16>()];
        let written = unsafe {
            opus_decode(
                self.state,
                encoded.as_ptr(),
                encoded.len().try_into().expect("data is out of range"),
                decoded.as_mut_ptr(),
                20 * self.sample_rate / 1000 * self.channels,
                fec.into(),
            )
        };

        if written < 0 {
            return Err(OpusError { error: written });
        }

        Ok(decoded)
    }
}
