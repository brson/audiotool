use crate::types::SampleRate;
use crate::io::Buf;

pub struct SampleRateConverter {
    x: (),
}

impl SampleRateConverter {
    pub fn new(inrate: SampleRate, outrate: SampleRate) -> SampleRateConverter {
        todo!()
    }

    pub fn convert(&mut self, inbuf: &Buf) -> &Buf {
        todo!()
    }

    pub fn finalize(&mut self) -> &Buf {
        todo!()
    }
}
