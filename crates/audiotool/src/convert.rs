mod config {
    use std::path::PathBuf;
    use rx::serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    #[derive(Clone)]
    pub struct Config {
        pub reference_tracks_dir: PathBuf,
        pub reference_track_regex: String,
        pub out_root_dir: PathBuf,
        pub out_path_template: String,
        pub formats: Vec<Format>,
    }

    #[derive(Serialize, Deserialize)]
    #[derive(Clone)]
    pub enum FormatKind {
        Flac,
        Alac,
        Vorbis,
        Mp3,
        Aac,
    }

    #[derive(Serialize, Deserialize)]
    #[derive(Clone)]
    pub struct Format {
        pub kind: FormatKind,
        pub bit_depth: u32,
        pub sample_rate: u32,
    }
}

pub use config::*;

use rx::prelude::*;
use rx::rayon::{self, prelude::*};

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

#[derive(Debug)]
pub struct ConvertResult {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
}

pub fn spawn(config: Config) -> (
    SyncSender<Request>,
    Receiver<Response>,
) {
    let (in_tx, in_rx) = sync_channel(1);
    let (out_tx, out_rx) = sync_channel(1);

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

    thread::spawn({
        let cancel = cancel.clone();
        move || {
            for req in rx.iter() {
                match req {
                    Request::Cancel => {
                        cancel.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        }
    });

    WalkDir::new(&config.reference_tracks_dir)
        .into_iter()
        .par_bridge()
        .try_for_each(|entry| {
            convert_entry(
                &config, entry, &tx, &cancel,
            );

            if cancel.load(Ordering::SeqCst) {
                return None;
            }
            return Some(());
        });

    let _ = tx.send(Response::Done);
}

fn convert_entry(
    config: &Config,
    entry: Result<DirEntry, walkdir::Error>,
    tx: &SyncSender<Response>,
    cancel: &AtomicBool,
) {
    match convert_entry2(
        config,
        entry,
        tx,
        cancel,
    ) {
        Ok(()) => { },
        Err(err) => {
            tx.send(Response::NextResult(Err(err.into())));
            return;
        }
    }
}

fn convert_entry2(
    config: &Config,
    entry: Result<DirEntry, walkdir::Error>,
    tx: &SyncSender<Response>,
    cancel: &AtomicBool,
) -> AnyResult<()> {
    let entry = entry?;
    let in_path = entry.path();
    let relative_dir = in_path.strip_prefix(&config.reference_tracks_dir)?;
    let out_dir = config.out_root_dir.join(&relative_dir);
    todo!()
}

fn convert_file(
    in_file: &Path,
    out_file: &Path,
    out_formats: &[Format],
    cancel: &AtomicBool,
) -> AnyResult<Option<ConvertResult>> {
    todo!();

    if cancel.load(Ordering::SeqCst) {
        return Ok(None);
    }

    todo!();
}

use crate::types::{SampleRate, BitDepth};
use std::collections::BTreeMap;
use crate::io::{PcmReader, PcmWriter};
use crate::samplerate::SampleRateConverter;
use crate::bitdepth::BitDepthConverter;

struct FilePlan<'up> {
    cancel: &'up AtomicBool,
    tx: &'up SyncSender<Response>,
    in_file: &'up Path,
    out_files: BTreeMap<SampleRate, BTreeMap<BitDepth, Vec<OutFile>>>,
}

struct OutFile {
    path: PathBuf,
    format: Format,
}

struct OutFileWriter {
    path: PathBuf,
    writer: Box<dyn PcmWriter>,
}

impl<'up> FilePlan<'up> {
    fn new<'up_>(
        config: &Config,
        entry: Result<DirEntry, walkdir::Error>,
        tx: &'up_ SyncSender<Response>,
        cancel: &'up_ AtomicBool,
    ) -> FilePlan<'up_> {
        todo!()
    }

    fn run(&self) {
        let out_files: BTreeMap<
                SampleRate,
            (
                SampleRateConverter,
                BTreeMap<
                        BitDepth,
                    (
                        BitDepthConverter,
                        Vec<OutFileWriter>
                    )>
            )> = todo!();

        let sample_rates: Vec<(
            SampleRate,
            SampleRateConverter,
            Vec<(
                BitDepth,
                BitDepthConverter,
                Vec<OutFileWriter>
            )>
        )> = todo!();
        
        loop {
            sample_rates.par_iter_mut().try_for_each(|args| {
                let (
                    sample_rate,
                    sample_rate_converter,
                    bit_depths
                ) = args;

                bit_depths.par_iter_mut().try_for_each(|args| {
                    let (
                        bit_depth,
                        bit_depth_converter,
                        writers,
                    ) = args;

                    Some(())
                })
            });
        }
    }
}
