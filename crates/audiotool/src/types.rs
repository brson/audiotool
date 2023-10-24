use rx::prelude::*;
use rx::serde::{Serialize, Deserialize};

pub trait SampleFormat {
    type Type;
}

pub struct F32;

impl SampleFormat for F32 {
    type Type = f32;
}

pub enum Buf {
    Uninit,
    F32(Vec<f32>),
    I24(Vec<i32>),
    I16(Vec<i16>),
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
pub struct Encoding {
    pub format: Format,
    pub bit_depth: BitDepth,
    pub sample_rate: SampleRate,
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
pub enum Format {
    Wav,
    Flac,
    Vorbis,
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
pub enum BitDepth {
    F32,
    I24,
    I16,
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
pub enum SampleRate {
    K192,
    K48,
}
