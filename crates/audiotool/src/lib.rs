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
    use rx::rand::Rng;
    use rx::rand_pcg::Pcg64Mcg;
    use rand_distr::{Triangular, Distribution};

    pub struct BitDepthConverter {
        inbits: BitDepth,
        outbits: BitDepth,
        dither: bool,
        outbuf: Buf,
        rng: Pcg64Mcg,
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

            // "Note that PCG specifies a default value for the parameter"
            let default_pcg_state = 0xcafef00dd15ea5e5;

            BitDepthConverter {
                inbits, outbits, dither,
                outbuf: Buf::Uninit,
                rng: Pcg64Mcg::new(default_pcg_state),
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
                            let mut outbuf = self.outbuf.i16_mut();
                            outbuf.truncate(0);
                            if !self.dither {
                                outbuf.extend(inbuf.iter().copied().map(f32_to_i16));
                            } else {
                                outbuf.extend(
                                    inbuf.iter().copied()
                                        .map(|s| dither_f32_for_i16(s, &mut self.rng))
                                        .map(f32_to_i16)
                                );
                            }
                        }
                    }
                }
                Buf::I24(inbuf) => {
                    assert_eq!(self.inbits, BitDepth::I24);
                    match self.outbits {
                        BitDepth::F32 => {
                            assert!(!self.dither);
                            let mut outbuf = self.outbuf.f32_mut();
                            outbuf.truncate(0);
                            outbuf.extend(inbuf.iter().copied().map(i24_to_f32));
                        }
                        _ => todo!(),
                    }
                }
                Buf::I16(inbuf) => {
                    assert_eq!(self.inbits, BitDepth::I16);
                    match self.outbits {
                        BitDepth::F32 => {
                            assert!(!self.dither);
                            let mut outbuf = self.outbuf.f32_mut();
                            outbuf.truncate(0);
                            outbuf.extend(inbuf.iter().copied().map(i16_to_f32));
                        }
                        _ => todo!(),
                    }
                }
                _ => todo!(),
            }

            &self.outbuf
        }
    }

    fn f32_to_i24(input: f32) -> i32 {
        let i24_min: i32 = -(2 ^ 15);
        let i24_max: i32 = (2 ^ 15) - 1;
        let i24_min = i24_min as f32;
        let i24_max = i24_max as f32;
        debug_assert!(input >= i24_min);
        debug_assert!(input <= i24_max);

        let range = i24_max - i24_min;
        let res = (input + 1.0) / 2.0 * range - i24_max;
        debug_assert!(res >= i24_min as f32 && res <= i24_max as f32);
        res as i32
    }

    fn f32_to_i16(input: f32) -> i16 {
        let i16_min = i16::MIN as f32;
        let i16_max = i16::MAX as f32;

        let range = i16_max - i16_min;
        let res = (input + 1.0) / 2.0 * range - i16_max;
        debug_assert!(res >= i16_min && res <= i16_max);
        res as i16
    }

    fn dither_f32_for_i16(input: f32, rng: &mut impl Rng) -> f32 {
        // fixme this is just a guess at how to dither
        let triangular = Triangular::new(-1.0, 1.0, 0.0).expect(".");
        let dither = triangular.sample(rng);
        (input + dither).clamp(-1.0, 1.0)
    }

    fn i24_to_f32(input: i32) -> f32 {
        let i24_min: i32 = -(2 ^ 15);
        let i24_max: i32 = (2 ^ 15) - 1;
        debug_assert!(input >= i24_min);
        debug_assert!(input <= i24_max);

        let i24_min = i24_min as f32;
        let i24_max = i24_max as f32;
        let input = input as f32;

        let range = i24_max - i24_min;

        let res = (input - i24_min) / range * 2.0 - 1.0;
        debug_assert!(res >= -1.0 && res <= 1.0);
        res
    }

    fn i16_to_f32(input: i16) -> f32 {
        let i16_min = i16::MIN as f32;
        let i16_max = i16::MAX as f32;
        let input = input as f32;

        let range = i16_max - i16_min;

        let res = (input - i16_min) / range - 1.0;
        debug_assert!(res >= -1.0 && res <= 1.0);
        res
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
