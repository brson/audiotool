use rx::prelude::*;
use rx::serde::{Serialize, Deserialize};

pub trait SampleFormat {
    type Type;
}

pub struct F32;

impl SampleFormat for F32 {
    type Type = f32;
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Format {
    pub codec: Codec,
    pub bit_depth: BitDepth,
    pub sample_rate: SampleRate,
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum Codec {
    Wav,
    Flac,
    Vorbis,
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum BitDepth {
    F32,
    I24,
    I16,
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum SampleRate {
    K192,
    K48,
}

impl SampleRate {
    pub fn as_u32(&self) -> u32 {
        match self {
            SampleRate::K192 => 192_000,
            SampleRate::K48 => 48_000,
        }
    }
}
