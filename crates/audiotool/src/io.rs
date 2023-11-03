use rx::prelude::*;
use crate::types::*;

pub enum Buf {
    Uninit,
    F32(Vec<f32>),
    I24(Vec<i32>),
    I16(Vec<i16>),
}

pub trait PcmReader: Send {
    fn props(&mut self) -> AnyResult<Format>;

    fn read(
        &mut self,
        buf: &mut Buf,
    ) -> AnyResult<()>;
}

pub trait PcmWriter: Send {
    fn props(&self) -> AnyResult<Format>;

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
