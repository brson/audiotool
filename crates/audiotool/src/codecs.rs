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

pub mod wav {
    use rx::prelude::*;
    use crate::types::{Format, BitDepth, SampleRate, Codec};
    use crate::io::{PcmReader, PcmWriter, Buf, Props};
    use std::path::Path;
    use std::io::{BufReader, BufWriter};
    use std::fs::File;

    pub struct WavPcmReader {
        reader: hound::Result<hound::WavReader<BufReader<File>>>,
    }

    impl WavPcmReader {
        pub fn new(path: &Path) -> WavPcmReader {
            WavPcmReader {
                reader: hound::WavReader::open(path),
            }
        }
    }

    impl PcmReader for WavPcmReader {
        fn props(&mut self) -> AnyResult<Props> {
            let reader = self.reader.as_ref()
                .map_err(|e| anyhow!("{e}"))?;
            let spec = reader.spec();
            Ok(Props {
                channels: spec.channels,
                format: Format {
                    codec: Codec::Wav,
                    bit_depth: match (spec.bits_per_sample, spec.sample_format) {
                        (32, hound::SampleFormat::Float) => BitDepth::F32,
                        (24, hound::SampleFormat::Int) => BitDepth::I24,
                        (16, hound::SampleFormat::Int) => BitDepth::I16,
                        (bits, format) => bail!("unsupported sample format: {bits}/{format:?}"),
                    },
                    sample_rate: match spec.sample_rate {
                        48_000 => SampleRate::K48,
                        192_000 => SampleRate::K192,
                        r => bail!("unsupported sample rate: {r} hz"),
                    }
                }
            })
        }

        fn read(
            &mut self,
            buf: &mut Buf,
        ) -> AnyResult<()> {
            let props = self.props()?;
            let reader = self.reader.as_mut()
                .map_err(|e| anyhow!("{e}"))?;
            match props.format.bit_depth {
                BitDepth::F32 => {
                    let bytes_to_read = 4096 * props.channels as usize;
                    let mut buf = buf.f32_mut();
                    buf.truncate(0);
                    buf.reserve_exact(bytes_to_read);
                    let mut samples = reader.samples::<f32>();
                    for _ in 0..bytes_to_read {
                        match samples.next() {
                            Some(sample) => {
                                buf.push(sample?);
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Ok(())
                }
                BitDepth::I24 => {
                    let bytes_to_read = 4096 * props.channels as usize;
                    let mut buf = buf.i24_mut();
                    buf.truncate(0);
                    buf.reserve_exact(bytes_to_read);
                    let mut samples = reader.samples::<i32>();
                    for _ in 0..bytes_to_read {
                        match samples.next() {
                            Some(sample) => {
                                buf.push(sample?);
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Ok(())
                }
                BitDepth::I16 => {
                    let bytes_to_read = 4096 * props.channels as usize;
                    let mut buf = buf.i16_mut();
                    buf.truncate(0);
                    buf.reserve_exact(bytes_to_read);
                    let mut samples = reader.samples::<i16>();
                    for _ in 0..bytes_to_read {
                        match samples.next() {
                            Some(sample) => {
                                buf.push(sample?);
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Ok(())
                }
            }            
        }
    }

    pub struct WavPcmWriter {
        writer: Option<hound::Result<hound::WavWriter<BufWriter<File>>>>,
    }

    impl WavPcmWriter {
        pub fn new(
            path: &Path,
            props: Props,
        ) -> WavPcmWriter {
            assert_eq!(props.format.codec, Codec::Wav);
            let spec = hound::WavSpec {
                channels: props.channels,
                sample_rate: props.format.sample_rate.as_u32(),
                bits_per_sample: match props.format.bit_depth {
                    BitDepth::F32 => 32,
                    BitDepth::I24 => 24,
                    BitDepth::I16 => 16,
                },
                sample_format: match props.format.bit_depth {
                    BitDepth::F32 => hound::SampleFormat::Float,
                    BitDepth::I24 => hound::SampleFormat::Int,
                    BitDepth::I16 => hound::SampleFormat::Int,
                },
            };
            WavPcmWriter {
                writer: Some(hound::WavWriter::create(path, spec)),
            }
        }
    }

    impl PcmWriter for WavPcmWriter {
        fn write(
            &mut self,
            buf: &Buf,
        ) -> AnyResult<()> {
            match &mut self.writer {
                Some(writer) => {
                    let writer = writer.as_mut()
                        .map_err(|e| anyhow!("{e}"))?;
                    match buf {
                        Buf::F32(buf) => {
                            for sample in buf.iter().copied() {
                                writer.write_sample(sample)?;
                            }
                        }
                        Buf::I24(buf) => {
                            for sample in buf.iter().copied() {
                                writer.write_sample(sample)?;
                            }
                        }
                        Buf::I16(buf) => {
                            for sample in buf.iter().copied() {
                                writer.write_sample(sample)?;
                            }
                        }
                        Buf::Uninit => panic!(),
                    }
                    Ok(())
                }
                None => {
                    panic!("already finalized");
                }
            }
        }

        fn finalize(&mut self) -> AnyResult<()> {
            let writer = std::mem::replace(&mut self.writer, None);
            match writer {
                Some(writer) => {
                    let writer = writer
                        .map_err(|e| anyhow!("{e}"))?;
                    writer.finalize()?;
                    Ok(())
                }
                None => {
                    panic!("already finalized");
                }
            }
        }
    }
}

pub mod flac {
    use rx::prelude::*;
    use crate::types::{Format, BitDepth, SampleRate, Codec};
    use crate::io::{PcmReader, PcmWriter, Buf, Props};
    use std::path::Path;
    use std::io::{BufReader, BufWriter};
    use std::fs::File;

    pub struct FlacPcmReader {
    }

    impl FlacPcmReader {
        pub fn new(path: &Path) -> FlacPcmReader {
            FlacPcmReader {
            }
        }
    }

    impl PcmReader for FlacPcmReader {
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

    pub struct FlacPcmWriter {
    }

    impl FlacPcmWriter {
        pub fn new(
            path: &Path,
            props: Props,
        ) -> FlacPcmWriter {
            todo!()
        }
    }

    impl PcmWriter for FlacPcmWriter {
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

pub mod vorbis {
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
}
