use rx::prelude::*;
use audiotool::types::*;
use audiotool::io::Props;
use audiotool::testsupport::*;

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
            sample_rate: SampleRate::K192,
        },
    )
}
