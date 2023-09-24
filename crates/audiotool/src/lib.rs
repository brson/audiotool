pub struct Config {
    pub reference_tracks_dir: PathBuf,
    pub reference_track_format: Format,
    pub outputs: Vec<OutDesc>,
}

pub struct OutDesc {
    
}

pub enum Format {
    Flac,
    Alac,
    Ogg,
    Mp3,
    Aac,
}
