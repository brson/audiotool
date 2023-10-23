#![allow(unused)]

pub mod convert;
pub mod split;

pub mod io {
    use rx::prelude::*;
    use crate::types::*;

    pub trait PcmReader: Send {
        fn props(&mut self) -> AnyResult<FileProps>;

        fn read(
            &mut self,
            buf: &mut Buf,
        ) -> AnyResult<()>;
    }

    pub trait PcmWriter: Send {
        fn props(&self) -> AnyResult<FileProps>;

        fn write(
            &mut self,
            buf: &Buf,
        ) -> AnyResult<()>;
    }

    fn static_assertions(
        reader: &dyn PcmReader,
        writer: &dyn PcmWriter,
    ) { }
}

pub mod types {
    use rx::prelude::*;

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

    pub struct FileProps {
        pub format: Format,
        pub bit_depth: BitDepth,
        pub sample_rate: SampleRate,
    }

    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    pub enum Format {
        Wav,
        Flac,
        Vorbis,
    }

    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    pub enum BitDepth {
        F32,
        I24,
        I16,
    }

    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    pub enum SampleRate {
        K192,
        K48,
    }
}

pub mod codecs {
    pub mod wav {
    }

    pub mod flac {
    }

    pub mod vorbis {
    }
}

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
