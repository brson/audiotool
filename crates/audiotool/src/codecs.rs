use rx::prelude::*;
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
            Box::new(wav::WavPcmReader::new(path))
        }
        _ => {
            todo!()
        }
    }
}

pub fn writer(
    path: &Path,
    channels: u16,
    format: Format
) -> Box<dyn PcmWriter> {
    match format.codec {
        Codec::Wav => {
            Box::new(wav::WavPcmWriter::new(path, channels, format.bit_depth, format.sample_rate))
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
        writer: hound::Result<hound::WavWriter<BufWriter<File>>>,
    }

    impl WavPcmWriter {
        pub fn new(
            path: &Path,
            channels: u16,
            bit_depth: BitDepth,
            sample_rate: SampleRate,
        ) -> WavPcmWriter {
            let spec = hound::WavSpec {
                channels,
                sample_rate: match sample_rate {
                    SampleRate::K192 => 192_000,
                    SampleRate::K48 => 48_000,
                },
                bits_per_sample: match bit_depth {
                    BitDepth::F32 => 32,
                    BitDepth::I24 => 24,
                    BitDepth::I16 => 16,
                },
                sample_format: match bit_depth {
                    BitDepth::F32 => hound::SampleFormat::Float,
                    BitDepth::I24 => hound::SampleFormat::Int,
                    BitDepth::I16 => hound::SampleFormat::Int,
                },
            };
            WavPcmWriter {
                writer: hound::WavWriter::create(path, spec),
            }
        }
    }

    impl PcmWriter for WavPcmWriter {
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
