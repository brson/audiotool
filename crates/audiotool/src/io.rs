use rx::prelude::*;
use crate::types::*;

#[derive(PartialEq, Debug)]
pub enum Buf {
    Uninit,
    F32(Vec<f32>),
    I24(Vec<i32>),
    I16(Vec<i16>),
}

#[derive(Eq, PartialEq, Debug)]
pub struct Props {
    pub channels: u16,
    pub format: Format,
}

pub trait PcmReader: Send {
    fn props(&mut self) -> AnyResult<Props>;

    fn read(
        &mut self,
        buf: &mut Buf,
    ) -> AnyResult<()>;
}

pub trait PcmWriter: Send {
    fn write(
        &mut self,
        buf: &Buf,
    ) -> AnyResult<()>;

    fn finalize(&mut self) -> AnyResult<()>;
}

fn static_assertions(
    reader: &dyn PcmReader,
    writer: &dyn PcmWriter,
) { }

impl Buf {
    pub fn is_empty(&self) -> bool {
        match self {
            Buf::Uninit => true,
            Buf::F32(v) => v.is_empty(),
            Buf::I24(v) => v.is_empty(),
            Buf::I16(v) => v.is_empty(),
        }
    }
}

pub struct PanicPcmWriter;

impl PcmWriter for PanicPcmWriter {
    fn write(
        &mut self,
        buf: &Buf,
    ) -> AnyResult<()> { panic!() }

    fn finalize(&mut self) -> AnyResult<()> { panic!() }
}

impl Buf {
    pub fn f32_mut(&mut self) -> &mut Vec<f32> {
        match self {
            Buf::F32(buf) => buf,
            _ => {
                *self = Buf::F32(vec![]);
                self.f32_mut()
            }
        }
    }

    pub fn i24_mut(&mut self) -> &mut Vec<i32> {
        match self {
            Buf::I24(buf) => buf,
            _ => {
                *self = Buf::I24(vec![]);
                self.i24_mut()
            }
        }
    }

    pub fn i16_mut(&mut self) -> &mut Vec<i16> {
        match self {
            Buf::I16(buf) => buf,
            _ => {
                *self = Buf::I16(vec![]);
                self.i16_mut()
            }
        }
    }
}
