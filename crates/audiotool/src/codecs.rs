use std::path::Path;
use crate::io::*;
use crate::types::*;

pub fn reader(path: &Path) -> Box<dyn PcmReader> {
    todo!()
}

pub fn writer(path: &Path, props: FileProps) -> Box<dyn PcmReader> {
    todo!()
}

pub mod wav {
}

pub mod flac {
}

pub mod vorbis {
}
