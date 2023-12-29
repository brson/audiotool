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
            (BitDepth::I24, BitDepth::F32, BitDepth::I24) => false,
            (BitDepth::I16, BitDepth::F32, BitDepth::I16) => false,
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

    pub fn convert<'a>(&'a mut self, inbuf: &'a Buf) -> &'a Buf {
        match inbuf {
            Buf::Uninit => panic!(),
            i @ Buf::F32(inbuf) => {
                assert_eq!(self.inbits, BitDepth::F32);
                match self.outbits {
                    BitDepth::F32 => {
                        assert!(!self.dither);
                        return i;
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

pub const I24_MAX: i32 = (2_i32.pow(23)) - 1;
pub const I24_MIN: i32 = -(2_i32.pow(23));
//pub const I24_MAX: i32 = (1_i32 << 23) - 1;
//pub const I24_MIN: i32 = -(1_i32 << 23);

pub fn i24_to_f32(input: i32) -> f32 {
    debug_assert!(input >= I24_MIN);
    debug_assert!(input <= I24_MAX);

    let i24_min = I24_MIN as f64;
    let i24_max = I24_MAX as f64;
    let input = input as f64;

    let range = i24_max - i24_min;

    //let res = (input - i24_min) / range * 2.0 - 1.0;
    let res = (input + 0.5) / (range / 2.0);
    debug_assert!(res >= -1.0 && res <= 1.0);
    res as f32
}

pub fn f32_to_i24(input: f32) -> i32 {
    let input = input as f64;
    let i24_min = I24_MIN as f64;
    let i24_max = I24_MAX as f64;
    debug_assert!(input >= -1.0);
    debug_assert!(input <= 1.0);

    let range = i24_max - i24_min;
    //let res = (input + 1.0) / 2.0 * range + i24_min;
    let res = (input * (range / 2.0)) - 0.5;
    debug_assert!(res >= i24_min as f64 && res <= i24_max as f64);
    // fixme rounding required here or not?
    res.round() as i32
}

pub fn i16_to_f32(input: i16) -> f32 {
    let i16_min = i16::MIN as f32;
    let i16_max = i16::MAX as f32;
    let input = input as f32;

    let range = i16_max - i16_min;

    //let res = (input - i16_min) / range * 2.0 - 1.0;
    let res = (input + 0.5) / (range / 2.0);
    debug_assert!(res >= -1.0 && res <= 1.0);
    res
}

pub fn f32_to_i16(input: f32) -> i16 {
    let i16_min = i16::MIN as f32;
    let i16_max = i16::MAX as f32;
    debug_assert!(input >= -1.0);
    debug_assert!(input <= 1.0);

    let range = i16_max - i16_min;
    //let res = (input + 1.0) / 2.0 * range + i16_min;
    let res = (input * (range / 2.0)) - 0.5;
    debug_assert!(res >= i16_min && res <= i16_max);
    // fixme unclear why rounding is required here
    res.round() as i16
}

pub fn i16_to_i24(input: i16) -> i32 {
    f32_to_i24(i16_to_f32(input))
}

pub fn i24_to_i16(input: i32) -> i16 {
    f32_to_i16(i24_to_f32(input))
}

// fixme doesn't produce same result as the fp conversion
pub fn i16_to_i24_no_fp(input: i16) -> i32 {
    if input < 0 {
        (input as i32) << 8
    } else {
        let output = (input as i32) << 8;
        output | 0xFF
    }
    //(input as i32) * 256
}

// todo test that this is the same as i24_to_i16
pub fn i24_to_i16_no_fp(input: i32) -> i16 {
    debug_assert!(input >= I24_MIN);
    debug_assert!(input <= I24_MAX);

    (input >> 8) as i16
}

// fixme this is just a guess
pub fn dither_f32_for_i16(input: f32, rng: &mut impl Rng) -> f32 {
    let i16_min = i16::MIN as f32;
    let i16_max = i16::MAX as f32;
    let i16_min = i16_min as f32;
    let i16_max = i16_max as f32;
    let range = i16_max - i16_min;

    let scaled_int_1 = 2.0 / range;

    let triangular = Triangular::new(
        -scaled_int_1,
        scaled_int_1,
        0.0
    ).expect(".");
    let dither = triangular.sample(rng);
    (input + dither).clamp(-1.0, 1.0)
}
