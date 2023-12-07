use rx::prelude::*;
use audiotool::convert as cvt;
use audiotool::types::*;
use audiotool::io::{Props, Buf};
use std::path::Path;

fn write_test_file(
    path: &Path,
    props: &Props,
    frames: u32,
) -> AnyResult<Buf> {
    todo!()
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
        out_path_template: S!("{{out_root_dir}}/{{relative_path}}/{{file_stem}}.{{format_ext}}"),
        formats: vec![
            Format {
                codec: Codec::Wav,
                bit_depth: BitDepth::F32,
                sample_rate: SampleRate::K48,
            },
        ]
    };

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

    let inbuf = write_test_file(&infile, &inprops, 1024)?;
    run_convert(config)?;
    let (outprops, outbuf) = read_file(&outfile)?;

    assert_eq!(inprops, outprops);
    assert_eq!(inbuf, outbuf);

    Ok(())
}
