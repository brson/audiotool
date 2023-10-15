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

pub use config::*;

use rx::prelude::*;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use std::thread;
use std::path::PathBuf;

pub enum Msg {
    GetNextResult,
    Cancel,
    Done,
}

pub enum Response {
    NextResult(AnyResult<ConvertResult>),
    Done,
}

pub struct ConvertResult {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
}

pub fn run(config: Config) -> (
    SyncSender<Msg>,
    Receiver<Response>,
) {
    let (in_tx, in_rx) = sync_channel(2);
    let (out_tx, out_rx) = sync_channel(2);

    thread::spawn(|| {
    });

    (in_tx, out_rx)
}
