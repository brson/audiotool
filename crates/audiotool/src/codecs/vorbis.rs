use rx::prelude::*;
use crate::types::{Format, BitDepth, SampleRate, Codec};
use crate::io::{PcmReader, PcmWriter, Buf, Props};
use std::path::Path;
use std::io::{BufReader, BufWriter};
use std::fs::File;

pub struct VorbisPcmReader {
}

impl VorbisPcmReader {
    pub fn new(path: &Path) -> VorbisPcmReader {
        VorbisPcmReader {
        }
    }
}

impl PcmReader for VorbisPcmReader {
    fn props(&mut self) -> AnyResult<Props> {
        todo!()
    }

    fn read(
        &mut self,
        buf: &mut Buf,
    ) -> AnyResult<()> {
        todo!()
    }
}

pub struct VorbisPcmWriter {
}

impl VorbisPcmWriter {
    pub fn new(
        path: &Path,
        props: Props,
    ) -> VorbisPcmWriter {
        assert_eq!(props.format.codec, Codec::Vorbis);
        todo!()
    }
}

impl PcmWriter for VorbisPcmWriter {
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
