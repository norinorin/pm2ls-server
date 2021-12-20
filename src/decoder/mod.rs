extern crate audiopus_sys;

use audiopus_sys::{opus_decode, opus_decode_float, opus_decoder_create, OPUS_OK};
use audiopus_sys::{opus_decoder_destroy, OpusDecoder as OpusDecoderState};

mod errors;

use errors::get_opus_error;
use errors::OpusError;

const IDEAL_FRAME_DURATION: u8 = 20;

pub struct OpusDecoder {
    state: *mut OpusDecoderState,
    sample_rate: i32,
}

impl OpusDecoder {
    pub fn new(sample_rate: i32, channels: i32) -> Result<Self, OpusError> {
        let mut error: i32 = OPUS_OK;
        let state: *mut OpusDecoderState =
            unsafe { opus_decoder_create(sample_rate, channels, &mut error) };

        if error != 0 {
            return Err(get_opus_error(error));
        }

        Ok(Self { state, sample_rate })
    }

    pub fn destroy(&self) {
        unsafe { opus_decoder_destroy(self.state) }
    }

    pub fn decode_float(&self, encoded: &[u8], fec: bool) -> Result<Vec<f32>, OpusError> {
        let frame_size = IDEAL_FRAME_DURATION as i32 * self.sample_rate / 1000;
        let mut decoded = vec![0.0; frame_size as usize];
        let written = unsafe {
            opus_decode_float(
                self.state,
                encoded.as_ptr(),
                encoded.len().try_into().expect("data is out of range"),
                decoded.as_mut_ptr(),
                frame_size,
                fec.into(),
            )
        };

        if written < 0 {
            return Err(get_opus_error(written));
        }

        Ok(decoded)
    }

    pub fn decode(&self, encoded: &[u8], fec: bool) -> Result<Vec<i16>, OpusError> {
        let frame_size = IDEAL_FRAME_DURATION as i32 * self.sample_rate / 1000;
        let mut decoded = vec![0; frame_size as usize];
        let written = unsafe {
            opus_decode(
                self.state,
                encoded.as_ptr(),
                encoded.len().try_into().expect("data is out of range"),
                decoded.as_mut_ptr(),
                frame_size,
                fec.into(),
            )
        };

        if written < 0 {
            return Err(get_opus_error(written));
        }

        Ok(decoded)
    }
}

impl Drop for OpusDecoder {
    fn drop(&mut self) {
        self.destroy();
    }
}
