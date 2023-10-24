use rx::prelude::*;
use crate::types::*;

pub trait PcmReader: Send {
    fn props(&mut self) -> AnyResult<Encoding>;

    fn read(
        &mut self,
        buf: &mut Buf,
    ) -> AnyResult<()>;
}

pub trait PcmWriter: Send {
    fn props(&self) -> AnyResult<Encoding>;

    fn write(
        &mut self,
        buf: &Buf,
    ) -> AnyResult<()>;
}

fn static_assertions(
    reader: &dyn PcmReader,
    writer: &dyn PcmWriter,
) { }
