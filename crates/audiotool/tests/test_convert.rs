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
use audiotool::testsupport::*;

#[test]
fn basic() -> AnyResult<()> {
    let tempdir = rx::tempfile::TempDir::with_prefix("audiotool")?;
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
