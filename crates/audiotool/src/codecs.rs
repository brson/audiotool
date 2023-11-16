use std::path::Path;
use crate::io::*;
use crate::types::*;
use std::ffi::OsStr;

pub fn reader(path: &Path) -> Box<dyn PcmReader> {
    let ext = path.extension()
        .map(OsStr::to_string_lossy)
        .as_deref()
        .map(str::to_string);
    match ext.as_deref() {
        Some("wav") => {
            Box::new(wav::WavPcmReader::new(path))
        }
        _ => {
            todo!()
        }
    }
}

pub fn writer(path: &Path, format: Format) -> Box<dyn PcmWriter> {
    match format.codec {
        Codec::Wav => {
            Box::new(wav::WavPcmWriter::new(path, format.bit_depth, format.sample_rate))
        }
        Codec::Flac => {
            todo!()
        }
        Codec::Vorbis => {
            todo!()
        }
    }
}

pub mod wav {
    use rx::prelude::*;
    use crate::types::{Format, BitDepth, SampleRate};
    use crate::io::{PcmReader, PcmWriter, Buf};
    use std::path::Path;

    pub struct WavPcmReader {
    }

    impl WavPcmReader {
        pub fn new(path: &Path) -> WavPcmReader {
            todo!()
        }
    }

    impl PcmReader for WavPcmReader {
        fn props(&mut self) -> AnyResult<Format> {
            todo!()
        }

        fn read(
            &mut self,
            buf: &mut Buf,
        ) -> AnyResult<()> {
            todo!()
        }
    }

    pub struct WavPcmWriter {
    }

    impl WavPcmWriter {
        pub fn new(
            path: &Path,
            bit_depth: BitDepth,
            sample_rate: SampleRate,
        ) -> WavPcmWriter {
            todo!()
        }
    }

    impl PcmWriter for WavPcmWriter {
        fn props(&self) -> AnyResult<Format> {
            todo!()
        }

        fn write(
            &mut self,
            buf: &Buf,
        ) -> AnyResult<()> {
            todo!()
        }

        fn finalize(&mut self) -> AnyResult<()> {
            todo!()
        }
    }
}

pub mod flac {
}

pub mod vorbis {
}
