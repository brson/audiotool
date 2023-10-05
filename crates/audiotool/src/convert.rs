mod config {
    use std::path::PathBuf;

    pub struct Config {
        pub reference_tracks_dir: PathBuf,
        pub reference_track_regex: String,
        pub out_root_dir: PathBuf,
        pub outputs: Vec<OutDesc>,
    }

    pub struct OutDesc {
        pub dir: PathBuf,
        pub format: Format,
    }

    pub enum Format {
        Flac(FlacFormat),
        Alac,
        Vorbis,
        Mp3,
        Aac,
    }

    pub struct FlacFormat {
        bit_depth: u32,
        sample_rate: u32,
    }
}

mod plan {
}
