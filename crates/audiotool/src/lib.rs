#![allow(unused)]

pub mod convert;
pub mod split;
pub mod io;
pub mod types;
pub mod codecs;

pub mod samplerate {
    pub struct SampleRateConverter;
}

pub mod bitdepth {
    pub struct BitDepthConverter;

    pub fn f32_to_i24(
        inbuf: &[f32],
        outbuf: &mut [i32],
    ) {
        todo!()
    }

    pub fn f32_to_i16(
        inbuf: &[f32],
        outbuf: &mut [i16],
    ) {
        todo!()
    }

    pub fn i24_to_i16(
        inbuf: &[i32],
        outbuf: &mut [i16],
    ) {
        todo!()
    }
}

pub mod dither {
    pub fn i24(
        inbuf: &[i32],
        outbuf: &mut [i32],
    ) {
        todo!()
    }
}
