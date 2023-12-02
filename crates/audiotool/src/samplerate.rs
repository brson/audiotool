use crate::types::SampleRate;
use crate::io::Buf;
use libsamplerate_sys::*;
use rx::libc::c_int;

pub struct SampleRateConverter {
    st: *mut SRC_STATE,
}

unsafe impl Send for SampleRateConverter {}

impl SampleRateConverter {
    pub fn new(inrate: SampleRate, outrate: SampleRate, channels: u16) -> SampleRateConverter {
        let mut error = 0;
        let st = unsafe {
            src_new(
                SRC_SINC_BEST_QUALITY as c_int,
                channels as c_int,
                &mut error,
            )
        };

        SampleRateConverter { st }
    }

    pub fn convert(&mut self, inbuf: &Buf) -> &Buf {
        todo!()
    }

    pub fn finalize(&mut self) -> &Buf {
        todo!()
    }
}

impl Drop for SampleRateConverter {
    fn drop(&mut self) {
    }
}
