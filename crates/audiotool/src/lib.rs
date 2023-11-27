#![allow(unused)]

pub mod convert;
pub mod split;
pub mod io;
pub mod types;
pub mod codecs;

pub mod samplerate {
    use crate::types::SampleRate;
    use crate::io::Buf;

    pub struct SampleRateConverter;

    impl SampleRateConverter {
        pub fn new(inrate: SampleRate, outrate: SampleRate) -> SampleRateConverter {
            todo!()
        }

        pub fn convert(&mut self, inbuf: &Buf) -> &Buf {
            todo!()
        }

        pub fn finalize(&mut self) -> &Buf {
            todo!()
        }
    }
}

pub mod bitdepth {
    use crate::types::BitDepth;
    use crate::io::Buf;

    pub struct BitDepthConverter {
        dither: bool,
    }

    impl BitDepthConverter {
        pub fn new(inbits: BitDepth, outbits: BitDepth, origbits: BitDepth) -> BitDepthConverter {
            todo!()
        }

        pub fn convert(&mut self, inbuf: &Buf) -> &Buf {
            todo!()
        }
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
