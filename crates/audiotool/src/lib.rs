#![allow(unused)]

pub mod convert;
pub mod split;
pub mod io;
pub mod types;
pub mod codecs;

pub mod samplerate {
    use crate::types::SampleRate;
    use crate::io::Buf;

    pub struct SampleRateConverter {
        x: (),
    }

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
        inbits: BitDepth,
        outbits: BitDepth,
        dither: bool,
        outbuf: Buf,
    }

    impl BitDepthConverter {
        pub fn new(inbits: BitDepth, outbits: BitDepth, origbits: BitDepth) -> BitDepthConverter {
            let dither = match (inbits, outbits, origbits) {
                (BitDepth::F32, BitDepth::F32, BitDepth::F32) => false,
                (BitDepth::F32, BitDepth::I24, BitDepth::F32) => false,
                (BitDepth::F32, BitDepth::I16, BitDepth::F32) => false,
                (BitDepth::F32, BitDepth::F32, BitDepth::I24) => false,
                (BitDepth::F32, BitDepth::I24, BitDepth::I24) => false,
                (BitDepth::F32, BitDepth::I16, BitDepth::I24) => true,
                (BitDepth::F32, BitDepth::F32, BitDepth::I16) => false,
                (BitDepth::F32, BitDepth::I24, BitDepth::I16) => false,
                (BitDepth::F32, BitDepth::I16, BitDepth::I16) => false,
                (_, _, _) => {
                    todo!()
                }
            };

            BitDepthConverter {
                inbits, outbits, dither,
                outbuf: Buf::Uninit,
            }
        }

        pub fn convert(&mut self, inbuf: &Buf) -> &Buf {
            match inbuf {
                Buf::Uninit => panic!(),
                Buf::F32(inbuf) => {
                    assert_eq!(self.inbits, BitDepth::F32);
                    match self.outbits {
                        BitDepth::F32 => {
                            assert!(!self.dither);
                            let mut outbuf = self.outbuf.f32_mut();
                            outbuf.truncate(0);
                            outbuf.extend(inbuf.iter());
                        }
                        BitDepth::I24 => {
                            assert!(!self.dither);
                            let mut outbuf = self.outbuf.i24_mut();
                            outbuf.truncate(0);
                            outbuf.extend(inbuf.iter().copied().map(f32_to_i24));
                        }
                        BitDepth::I16 => {
                            let mut outbuf = self.outbuf.i24_mut();
                            outbuf.truncate(0);
                            if !self.dither {
                                outbuf.extend(inbuf.iter().copied().map(f32_to_i24));
                            } else {
                                outbuf.extend(
                                    inbuf.iter().copied()
                                        .map(|s| dither_f32_for_i24(s))
                                        .map(f32_to_i24)
                                );
                            }
                        }
                    }
                }
                _ => todo!(),
            }

            &self.outbuf
        }
    }

    fn f32_to_i24(input: f32) -> i32 {
        todo!()
    }

    fn dither_f32_for_i24(input: f32) -> f32 {
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
