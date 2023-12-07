use crate::types::SampleRate;
use crate::io::Buf;
use libsamplerate_sys::*;
use rx::libc::{c_int, c_long};

pub struct SampleRateConverter {
    state: *mut SRC_STATE,
    outbuf: Buf,
    channels: u16,
    src_ratio: f64,
}

unsafe impl Send for SampleRateConverter {}

impl SampleRateConverter {
    pub fn new(inrate: SampleRate, outrate: SampleRate, channels: u16) -> SampleRateConverter {
        let mut error = 0;
        let state = unsafe {
            src_new(
                SRC_SINC_BEST_QUALITY as c_int,
                channels as c_int,
                &mut error,
            )
        };

        assert!(error == 0);
        assert!(!state.is_null());

        let inrate = inrate.as_u32() as f64;
        let outrate = outrate.as_u32() as f64;
        let src_ratio = outrate / inrate;

        SampleRateConverter {
            state,
            outbuf: Buf::Uninit,
            channels,
            src_ratio,
        }
    }

    pub fn convert<'a>(&'a mut self, inbuf: &'a Buf) -> &'a Buf {
        if self.src_ratio == 1.0 {
            return inbuf;
        }
        match inbuf {
            Buf::F32(inbuf) => {
                assert_eq!(inbuf.len() % self.channels as usize, 0);
                let expected_outbuf_size =
                    (
                        (inbuf.len() as f64 / (self.channels as f64) * self.src_ratio)
                            .ceil() * (self.channels as f64)
                    ) as usize;
                let mut outbuf = self.outbuf.f32_mut();
                outbuf.resize(expected_outbuf_size, 0.0);
                let mut data = SRC_DATA {
                    data_in: inbuf.as_ptr(),
                    data_out: outbuf.as_mut_ptr(),
                    input_frames: (inbuf.len() / self.channels as usize) as c_long,
                    output_frames: (expected_outbuf_size  / self.channels as usize) as c_long,
                    input_frames_used: 0,
                    output_frames_gen: 0,
                    end_of_input: 0,
                    src_ratio: self.src_ratio,
                };

                let err = unsafe { src_process(self.state, &mut data) };
                assert_eq!(err, 0);

                assert_eq!(data.input_frames_used, data.input_frames);
                let output_bytes_gen = data.output_frames_gen as usize * (self.channels as usize);
                outbuf.truncate(output_bytes_gen);
                
                &self.outbuf
            }
            _ => panic!(),
        }
    }

    pub fn finalize(&mut self) -> &Buf {
        if self.src_ratio == 1.0 {
            let mut buf = self.outbuf.f32_mut();
            buf.truncate(0);
            return &self.outbuf;
        }
        let expected_outbuf_size = 1 * self.channels as usize;
        let mut outbuf = self.outbuf.f32_mut();
        outbuf.resize(expected_outbuf_size, 0.0);
        let mut data = SRC_DATA {
            data_in: std::ptr::null(),
            data_out: outbuf.as_mut_ptr(),
            input_frames: 0,
            output_frames: (expected_outbuf_size  / self.channels as usize) as c_long,
            input_frames_used: 0,
            output_frames_gen: 0,
            end_of_input: 1,
            src_ratio: self.src_ratio,
        };

        let err = unsafe { src_process(self.state, &mut data) };
        assert_eq!(err, 0);

        assert_eq!(data.input_frames_used, data.input_frames);
        let output_bytes_gen = data.output_frames_gen as usize * (self.channels as usize);
        outbuf.truncate(output_bytes_gen);
        
        &self.outbuf
    }
}

impl Drop for SampleRateConverter {
    fn drop(&mut self) {
        unsafe { src_delete(self.state); }
    }
}
