use rx::prelude::*;
use rx::itertools::Itertools;
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
    const CHANNELS: &[u16] = &[1];
    //const CODECS: &[Codec] = &[Codec::Wav, Codec::Flac, Codec::Vorbis];
    const CODECS: &[Codec] = &[Codec::Flac];
    const BIT_DEPTHS: &[BitDepth] = &[BitDepth::I16];
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
    test_basic(test.inprops, test.outformat)
}
