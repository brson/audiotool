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
            todo!()
        }
        _ => {
            todo!()
        }
    }
}

pub fn writer(path: &Path, format: Format) -> Box<dyn PcmWriter> {
    match format.codec {
        Codec::Wav => {
            todo!()
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
    use crate::types::Format;
    use crate::io::{PcmReader, PcmWriter, Buf};

    pub struct WavPcmReader {
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
