use rx::prelude::*;
use audiotool::convert as cvt;
use audiotool::types::*;

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
                bit_depth: BitDepth::I24,
                sample_rate: SampleRate::K48,
            },
        ]
    };

    todo!()
}
