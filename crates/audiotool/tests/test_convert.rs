use rx::prelude::*;
use audiotool::convert as cvt;
use audiotool::types::*;
use audiotool::io::Props;
use audiotool::testsupport::*;

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

fn test_basic(
    inprops: Props,
    outformat: Format,
) -> AnyResult<()> {
    let tempdir = rx::tempfile::TempDir::with_prefix("audiotool")?;
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

#[test]
fn basic_wav_wav() -> AnyResult<()> {
    test_basic(
        Props {
            channels: 2,
            format: Format {
                codec: Codec::Wav,
                bit_depth: BitDepth::F32,
                sample_rate: SampleRate::K48,
            },
        },
        Format {
            codec: Codec::Wav,
            bit_depth: BitDepth::F32,
            sample_rate: SampleRate::K48,
        },
    )
}

#[test]
fn basic_wav_flac() -> AnyResult<()> {
    test_basic(
        Props {
            channels: 2,
            format: Format {
                codec: Codec::Wav,
                bit_depth: BitDepth::I24,
                sample_rate: SampleRate::K48,
            },
        },
        Format {
            codec: Codec::Flac,
            bit_depth: BitDepth::I24,
            sample_rate: SampleRate::K48,
        },
    )
}

#[test]
fn basic_flac_flac() -> AnyResult<()> {
    test_basic(
        Props {
            channels: 2,
            format: Format {
                codec: Codec::Flac,
                bit_depth: BitDepth::I24,
                sample_rate: SampleRate::K48,
            },
        },
        Format {
            codec: Codec::Flac,
            bit_depth: BitDepth::I24,
            sample_rate: SampleRate::K48,
        },
    )
}
