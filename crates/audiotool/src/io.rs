use rx::prelude::*;
use crate::types::*;

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
