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
use rx::rayon::prelude::*;

use rx::walkdir::{self, WalkDir, DirEntry};
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::path::{PathBuf, Path};

pub enum Request {
    Cancel,
}

pub enum Response {
    NextResult(AnyResult<ConvertResult>),
    Done,
}

pub struct ConvertResult {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
}

pub fn spawn(config: Config) -> (
    SyncSender<Request>,
    Receiver<Response>,
) {
    let (in_tx, in_rx) = sync_channel(2);
    let (out_tx, out_rx) = sync_channel(2);

    thread::spawn(move || {
        run(config, in_rx, out_tx)
    });

    (in_tx, out_rx)
}

fn run(
    config: Config,
    rx: Receiver<Request>,
    tx: SyncSender<Response>,
) {
    let cancel = Arc::new(AtomicBool::from(false));

    let join_handle = {
        let tx = tx.clone();
        let cancel = cancel.clone();
        thread::spawn(move || {
            WalkDir::new(&config.reference_tracks_dir)
                .into_iter()
                .par_bridge()
                .try_for_each(|entry| {
                    let keep_going = convert_entry(
                        &config, &entry, &tx, &cancel,
                    );

                    return keep_going;
                });
        })
    };

    thread::spawn(move || {
        for req in rx.iter() {
            match req {
                Request::Cancel => {
                    cancel.store(true, Ordering::SeqCst);
                    break;
                }
            }
        }
    });

    join_handle.join().expect("join");

    let _ = tx.send(Response::Done);
}

fn convert_entry(
    config: &Config,
    entry: &Result<DirEntry, walkdir::Error>,
    tx: &SyncSender<Response>,
    cancel: &AtomicBool,
) -> Option<()> {
    if cancel.load(Ordering::SeqCst) {
        return None;
    }

    todo!()
}

fn convert_file(
    in_file: &Path,
    out_file: &Path,
    out_format: Format,
    cancel: &AtomicBool,
) -> Option<AnyResult<ConvertResult>> {
    todo!()
}

