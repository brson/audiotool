use rx::prelude::*;
use rx::rand_pcg::Pcg64Mcg;
use rx::rand::Rng;
use rx::itertools::Itertools;
use std::path::Path;
use std::iter;
use audiotool::convert as cvt;
use audiotool::types::*;
use audiotool::io::{Props, Buf};
use audiotool::codecs;

fn write_test_file(
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
                    Some(rng.gen_range(-1.0..=1.0))
                }).take(samples).collect()
            )
        }
        _ => todo!(),
    };

    let mut writer = codecs::writer(path, props);
    writer.write(&buf)?;
    writer.finalize()?;

    Ok(buf)
}

fn read_file(path: &Path) -> AnyResult<(Props, Buf)> {
    #[extension_trait]
    impl BufExt for Buf {
        fn append(&mut self, other: &Buf) {
            match (self, other) {
                (this @ Buf::Uninit, Buf::F32(other)) => {
                    *this = Buf::F32(other.clone());
                }
                (Buf::F32(ref mut this), Buf::F32(other)) => {
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

fn run_convert(config: cvt::config::Config) -> AnyResult<()> {
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

#[test]
fn basic() -> AnyResult<()> {
    let tempdir = rx::tempfile::tempdir()?;
    let config = cvt::config::Config {
        reference_tracks_dir: tempdir.path().join("in"),
        reference_track_regex: S("\\.wav$"),
        out_root_dir: tempdir.path().join("out"),
        out_path_template: S("{{out_root_dir}}/{{relative_path}}/{{file_stem}}.{{format_ext}}"),
        formats: vec![
            Format {
                codec: Codec::Wav,
                bit_depth: BitDepth::F32,
                sample_rate: SampleRate::K48,
            },
        ]
    };

    std::fs::create_dir_all(&config.reference_tracks_dir)?;

    let inprops = Props {
        format: Format {
            codec: Codec::Wav,
            bit_depth: BitDepth::F32,
            sample_rate: SampleRate::K48,
        },
        channels: 2,
    };
    let infile = config.reference_tracks_dir.join("test.wav");
    let outfile = config.out_root_dir.join("test.wav");

    let frames = 1024;

    let inbuf = write_test_file(&infile, inprops, frames)?;
    run_convert(config)?;
    let (outprops, outbuf) = read_file(&outfile)?;

    assert_eq!(inprops, outprops);
    assert_eq!(inbuf, outbuf);

    Ok(())
}

struct SingleTestCase {
    inprops: Props,
    outformat: Format,
}

fn all_single_test_cases() -> impl Iterator<Item = SingleTestCase> {
    const CHANNELS: &[u16] = &[1, 2];
    const CODECS: &[Codec] = &[Codec::Wav, Codec::Flac, Codec::Vorbis];
    const BIT_DEPTHS: &[BitDepth] = &[BitDepth::F32, BitDepth::I24, BitDepth::I16];
    const SAMPLE_RATES: &[SampleRate] = &[SampleRate::K192, SampleRate::K48];

    let all_formats = || CODECS.iter().copied()
        .cartesian_product(BIT_DEPTHS.iter().copied())
        .cartesian_product(SAMPLE_RATES.iter().copied())
        .map(|((codec, bit_depth), sample_rate)| {
            Format {
                codec, bit_depth, sample_rate,
            }
        });

    let inprops = all_formats()
        .cartesian_product(CHANNELS.iter().copied())
        .map(|(format, channels)| {
            Props {
                channels, format,
            }
        });
    let outformats = all_formats();
    
    inprops.cartesian_product(outformats)
        .map(|(inprops, outformat)| {
            SingleTestCase {
                inprops, outformat,
            }
        })
}

fn run_single_test_case(test: SingleTestCase) -> AnyResult<()> {
    let tempdir = rx::tempfile::tempdir()?;
    let config = cvt::config::Config {
        reference_tracks_dir: tempdir.path().join("in"),
        reference_track_regex: S("\\.wav$"),
        out_root_dir: tempdir.path().join("out"),
        out_path_template: S("{{out_root_dir}}/{{relative_path}}/{{file_stem}}.{{format_ext}}"),
        formats: vec![
            test.outformat,
        ]
    };

    std::fs::create_dir_all(&config.reference_tracks_dir)?;

    let inprops = test.inprops;
    let infile = config.reference_tracks_dir.join("test.wav");
    let outfile = config.out_root_dir.join("test.wav");

    let frames = 1024;

    let inbuf = write_test_file(&infile, inprops, frames)?;
    run_convert(config)?;
    let (outprops, outbuf) = read_file(&outfile)?;

    assert_eq!(inprops, outprops);
    assert_eq!(inbuf, outbuf);

    Ok(())
}
