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
    let plan = FilePlan::new(
        config,
        entry,
        tx,
        cancel,
    );

    plan.run();
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
    sample_rates: BTreeMap<SampleRate, BTreeMap<BitDepth, Vec<OutFile>>>,
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
        let mut sample_rates: BTreeMap<SampleRate, BTreeMap<BitDepth, Vec<OutFile>>> = BTreeMap::new();

        for format in &config.formats {
        }

        todo!()
    }

    fn run(&self) {
        let mut sample_rates: BTreeMap<
                SampleRate,
            (
                SampleRateConverter,
                BTreeMap<
                        BitDepth,
                    (
                        BitDepthConverter,
                        Vec<Option<OutFileWriter>>
                    )>
            )> = self.sample_rates.iter().map(|args| {
                let (
                    sample_rate,
                    bit_depths,
                ) = args;

                let bit_depths = bit_depths.iter().map(|args| {
                    let (
                        bit_depth,
                        outfiles,
                    ) = args;

                    let writers = outfiles.iter().map(|outfile| {
                        todo!()
                    }).collect();

                    (
                        *bit_depth,
                        (
                            BitDepthConverter,
                            writers,
                        ),
                    )
                }).collect();

                (
                    *sample_rate,
                    (
                        SampleRateConverter,
                        bit_depths,
                    ),
                )
            }).collect();

        loop {
            if self.cancel.load(Ordering::SeqCst) {
                break;
            }

            // todo read next data;

            let keep_going = sample_rates.par_iter_mut().try_for_each(|args| {
                let (
                    sample_rate,
                    (
                        sample_rate_converter,
                        bit_depths,
                    ),
                ) = args;

                todo!();

                bit_depths.par_iter_mut().try_for_each(|args| {
                    let (
                        bit_depth,
                        (
                            bit_depth_converter,
                            writers,
                        ),
                    ) = args;

                    todo!();

                    writers.par_iter_mut().try_for_each(|writer| {

                        if self.cancel.load(Ordering::SeqCst) {
                            return None;
                        }

                        todo!();

                        Some(())
                    })
                })
            });

            if keep_going.is_none() {
                break;
            }
        }

        // Do cleanups and send cancellation errors.
        for (_, (_, bit_depths)) in sample_rates.into_iter() {
            for (_, (_, writers)) in bit_depths.into_iter() {
                for writer in writers {
                    if let Some(writer) = writer {
                        todo!()
                    }
                }
            }
        }
    }
}
