mod wav;
mod flac;
mod vorbis;

use rx::prelude::*;
use std::path::Path;
use crate::io::*;
use crate::types::*;
use std::ffi::OsStr;

pub fn reader(path: &Path) -> AnyResult<Box<dyn PcmReader>> {
    let ext = path.extension()
        .map(OsStr::to_string_lossy)
        .as_deref()
        .map(str::to_string);
    match ext.as_deref() {
        Some("wav") => {
            Ok(Box::new(wav::WavPcmReader::new(path)))
        }
        Some("flac") => {
            Ok(Box::new(flac::FlacPcmReader::new(path)))
        }
        Some("ogg") => {
            Ok(Box::new(vorbis::VorbisPcmReader::new(path)))
        }
        Some(ext) => {
            Err(anyhow!("unknown extension: `{ext}`"))
        }
        None => {
            Err(anyhow!("no file extension"))
        }
    }
}

pub fn writer(
    path: &Path,
    props: Props,
) -> Box<dyn PcmWriter> {
    match props.format.codec {
        Codec::Wav => {
            Box::new(wav::WavPcmWriter::new(path, props))
        }
        Codec::Flac => {
            Box::new(flac::FlacPcmWriter::new(path, props))
        }
        Codec::Vorbis => {
            Box::new(vorbis::VorbisPcmWriter::new(path, props))
        }
    }
}
