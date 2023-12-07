use rx::prelude::*;
use rx::rand_pcg::Pcg64Mcg;
use rx::rand::Rng;
use std::path::Path;
use std::iter;
use audiotool::convert as cvt;
use audiotool::types::*;
use audiotool::io::{Props, Buf, PcmReader, PcmWriter};
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
    todo!()
}

fn run_convert(config: cvt::config::Config) -> AnyResult<()> {
    let (tx, rx) = cvt::plan::spawn(config);
    let plan = match rx.recv().expect("recv") {
        cvt::plan::Response::Done(Ok(Some(plan))) => plan,
        cvt::plan::Response::Done(Ok(None)) => panic!(),
        cvt::plan::Response::Done(Err(e)) => panic!("{e}"),
    };
    
    let (tx, rx) = cvt::exec::spawn(plan);

    loop {
        let resp = rx.recv()?;

        match resp {
            cvt::exec::Response::NextResult(res) => {
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
        reference_track_regex: S!("\\.wav$"),
        out_root_dir: tempdir.path().join("out"),
        out_path_template: "{{out_root_dir}}/{{relative_path}}/{{file_stem}}.{{format_ext}}".to_string(),
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

    let channels = 2;
    let frames = 1024;

    let inbuf = write_test_file(&infile, inprops, 1024)?;
    run_convert(config)?;
    let (outprops, outbuf) = read_file(&outfile)?;

    assert_eq!(inprops, outprops);
    assert_eq!(inbuf, outbuf);

    Ok(())
}
