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

fn main() {
    use libtest_mimic::{Arguments, Trial};

    let args = Arguments::from_args();

    let tests = all_single_test_cases()
        .map(|test| {
            todo!()
        }).collect();

    libtest_mimic::run(&args, tests).exit();
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
