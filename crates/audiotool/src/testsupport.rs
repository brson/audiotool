use rmx::prelude::*;
use rmx::rand_pcg::Pcg64Mcg;
use rmx::rand::Rng;
use rmx::itertools::Itertools;
use std::path::Path;
use std::iter;
use crate::types::*;
use crate::io::{Props, Buf};
use crate::codecs;
use crate::bitdepth::{I24_MIN, I24_MAX};
use crate::convert as cvt;

pub fn write_test_file(
    path: &Path,
    props: Props,
    frames: u32,
) -> AnyResult<Buf> {
    let mut rng = Pcg64Mcg::new(0);
    let samples = frames as usize * props.channels as usize;
    let buf = match props.format.bit_depth {
        BitDepth::F32 => {
            Buf::F32(
                iter::from_fn(|| {
                    Some(rng.random_range(-1.0..=1.0))
                }).take(samples).collect()
            )
        }
        BitDepth::I24 => {
            Buf::I24(
                iter::from_fn(|| {
                    Some(rng.random_range(I24_MIN..=I24_MAX))
                }).take(samples).collect()
            )
        }
        BitDepth::I16 => {
            Buf::I16(
                iter::from_fn(|| {
                    Some(rng.random_range(i16::MIN..=i16::MAX))
                }).take(samples).collect()
            )
        }
    };

    let mut writer = codecs::writer(path, props);
    writer.write(&buf)?;
    writer.finalize()?;

    Ok(buf)
}

pub fn read_file(path: &Path) -> AnyResult<(Props, Buf)> {
    #[extension_trait]
    impl BufExt for Buf {
        fn append(&mut self, other: &Buf) {
            match (self, other) {
                (this @ Buf::Uninit, Buf::F32(other)) => {
                    *this = Buf::F32(other.clone());
                }
                (Buf::F32(this), Buf::F32(other)) => {
                    this.extend(other.iter());
                },
                (this @ Buf::Uninit, Buf::I24(other)) => {
                    *this = Buf::I24(other.clone());
                }
                (Buf::I24(this), Buf::I24(other)) => {
                    this.extend(other.iter());
                },
                (this @ Buf::Uninit, Buf::I16(other)) => {
                    *this = Buf::I16(other.clone());
                }
                (Buf::I16(this), Buf::I16(other)) => {
                    this.extend(other.iter());
                },
                 _ => todo!(),
            }
        }
    }

    let mut reader = codecs::reader(path)?;
    let mut all_buf = Buf::Uninit;
    let mut tmp_buf = Buf::Uninit;

    loop {
        reader.read(&mut tmp_buf)?;

        if tmp_buf.is_empty() {
            break;
        }

        all_buf.append(&tmp_buf);
    }

    Ok((reader.props()?, all_buf))
}

pub fn run_convert(config: cvt::config::Config) -> AnyResult<()> {
    let (_tx, rx) = cvt::plan::spawn(config);

    let plan = match rx.recv().expect("recv") {
        cvt::plan::Response::Done(Ok(Some(plan))) => plan,
        cvt::plan::Response::Done(Ok(None)) => panic!(),
        cvt::plan::Response::Done(Err(e)) => panic!("{e}"),
    };

    let (_tx, rx) = cvt::exec::spawn(plan);

    loop {
        let resp = rx.recv()?;

        match resp {
            cvt::exec::Response::NextResult(_res) => {
                //println!("{res:#?}");
            }
            cvt::exec::Response::Done => {
                break;
            }
            cvt::exec::Response::Cancelled => {
                panic!();
            }
        }
    }

    Ok(())
}

#[extension_trait]
impl CodecExt for Codec {
    fn ext(&self) -> &'static str {
        match self {
            Codec::Wav => "wav",
            Codec::Flac => "flac",
            Codec::Vorbis => "ogg",
        }
    }
}

pub fn test_basic(
    inprops: Props,
    outformat: Format,
) -> AnyResult<()> {
    let tempdir = rmx::tempfile::TempDir::with_prefix("audiotool")?;
    let config = cvt::config::Config {
        reference_tracks_dir: tempdir.path().join("in"),
        reference_track_regex: format!("\\.{}$", inprops.format.codec.ext()),
        out_root_dir: tempdir.path().join("out"),
        out_path_template: S("{{out_root_dir}}/{{relative_path}}/{{file_stem}}.{{format_ext}}"),
        formats: vec![outformat],
    };

    std::fs::create_dir_all(&config.reference_tracks_dir)?;

    let infile = config.reference_tracks_dir.join(format!("test.{}", inprops.format.codec.ext()));
    let outfile = config.out_root_dir.join(format!("test.{}", outformat.codec.ext()));

    let frames = 1024;

    let inbuf = write_test_file(&infile, inprops, frames)?;
    run_convert(config)?;
    let (outprops, outbuf) = read_file(&outfile)?;

    let expected_outprops = Props {
        channels: inprops.channels,
        format: outformat,
    };

    assert_eq!(expected_outprops, outprops);
    
    if inprops.format.bit_depth == outprops.format.bit_depth
        && inprops.format.sample_rate == outprops.format.sample_rate
    {
        assert_eq!(inbuf, outbuf);
    }

    if inprops.format.sample_rate == outprops.format.sample_rate {
        assert_eq!(inbuf.len(), outbuf.len());
    }

    if inprops.format.sample_rate > outprops.format.sample_rate {
        assert!(inbuf.len() < outbuf.len());
    }

    if inprops.format.sample_rate < outprops.format.sample_rate {
        assert!(inbuf.len() > outbuf.len());
    }

    Ok(())
}
