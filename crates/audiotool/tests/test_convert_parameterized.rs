use rx::prelude::*;
use rx::itertools::Itertools;
use audiotool::convert as cvt;
use audiotool::types::*;
use audiotool::io::{Props};
use audiotool::testsupport::*;

#[extension_trait]
impl FormatExt for Format {
    fn test_string(&self) -> String {
        format!(
            "{}_{}_{}",
            match self.codec {
                Codec::Wav => "wav",
                Codec::Flac => "flac",
                Codec::Vorbis => "vorbis",
            },
            match self.bit_depth {
                BitDepth::F32 => "f32",
                BitDepth::I24 => "i24",
                BitDepth::I16 => "i16",
            },
            match self.sample_rate {
                SampleRate::K192 => "k192",
                SampleRate::K48 => "k48",
            },
        )
    }
}

fn main() {
    use libtest_mimic::{Arguments, Trial};

    let args = Arguments::from_args();

    let tests = all_single_test_cases()
        .map(|test| {
            Trial::test(
                {
                    format!(
                        "{infmt}_to_{outfmt}_x{channels}",
                        infmt = test.inprops.format.test_string(),
                        outfmt = test.outformat.test_string(),
                        channels = test.inprops.channels,
                    )
                },
                || {
                    run_single_test_case(test)?;
                    Ok(())
                },
            )
        }).collect();

    libtest_mimic::run(&args, tests).exit();
}

#[derive(Debug)]
struct SingleTestCase {
    inprops: Props,
    outformat: Format,
}

fn all_single_test_cases() -> impl Iterator<Item = SingleTestCase> {
    //const CHANNELS: &[u16] = &[1, 2];
    const CHANNELS: &[u16] = &[1, 2];
    //const CODECS: &[Codec] = &[Codec::Wav, Codec::Flac, Codec::Vorbis];
    const CODECS: &[Codec] = &[Codec::Wav];
    //const BIT_DEPTHS: &[BitDepth] = &[BitDepth::F32, BitDepth::I24, BitDepth::I16];
    const BIT_DEPTHS: &[BitDepth] = &[BitDepth::F32, BitDepth::I24, BitDepth::I16];
    //const SAMPLE_RATES: &[SampleRate] = &[SampleRate::K192, SampleRate::K48];
    const SAMPLE_RATES: &[SampleRate] = &[SampleRate::K48];

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
    let tempdir = rx::tempfile::TempDir::with_prefix("audiotool")?;
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

    // todo parameterize
    let frames = 1024;

    let inbuf = write_test_file(&infile, inprops, frames)?;
    run_convert(config)?;
    let (outprops, outbuf) = read_file(&outfile)?;

    assert_eq!(outprops, Props {
        channels: inprops.channels,
        format: test.outformat,
    });

    if inprops.format.bit_depth == outprops.format.bit_depth
        && inprops.format.sample_rate == outprops.format.sample_rate
    {
        assert_eq!(inbuf, outbuf);
    }

    if inprops.format.sample_rate == outprops.format.sample_rate {
        assert_eq!(inbuf.len(), outbuf.len());
    }

    if inprops.format.sample_rate > outprops.format.sample_rate {
        assert!(inbuf.len() > outbuf.len());
    }

    if inprops.format.sample_rate < outprops.format.sample_rate {
        assert!(inbuf.len() < outbuf.len());
    }

    Ok(())
}
