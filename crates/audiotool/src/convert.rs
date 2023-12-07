pub mod config {
    use rx::prelude::*;
    use std::path::PathBuf;
    use rx::serde::{Serialize, Deserialize};
    use crate::types::{Format, Codec, BitDepth, SampleRate};

    #[derive(Serialize, Deserialize)]
    #[derive(Clone)]
    pub struct Config {
        pub reference_tracks_dir: PathBuf,
        pub reference_track_regex: String,
        pub out_root_dir: PathBuf,
        pub out_path_template: String,
        pub formats: Vec<Format>,
    }

    impl Config {
        pub fn template() -> Config {
            Config {
                reference_tracks_dir: S!("./in/").into(),
                reference_track_regex: S!("\\.wav"),
                out_root_dir: S!("./out/").into(),
                out_path_template: S!("{{out_root_dir}}/{{relative_path}}/{{file_stem}}.{{format_ext}}"),
                formats: vec![
                    Format {
                        codec: Codec::Wav,
                        bit_depth: BitDepth::I24,
                        sample_rate: SampleRate::K48,
                    },
                ]
            }
        }
    }
}

pub mod plan {
    use rx::prelude::*;
    use rx::rayon::{self, prelude::*};

    use super::config::Config;
    use crate::types::Format;
    use super::OutFile;

    use rx::walkdir::{self, WalkDir, DirEntry};
    use std::sync::mpsc::{SyncSender, Receiver, sync_channel, TryRecvError};
    use std::path::PathBuf;
    use std::thread;

    pub struct Plan {
        pub outputs: Vec<InfilePlan>,
    }

    pub struct InfilePlan {
        pub infile: PathBuf,
        pub outfiles: Vec<OutFile>,
    }

    pub enum Request {
        Cancel,
    }

    pub enum Response {
        Done(AnyResult<Option<Plan>>),
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
        let maybe_plan = run2(config, rx);
        let _ = tx.send(Response::Done(maybe_plan));
    }

    fn run2(
        config: Config,
        rx: Receiver<Request>,
    ) -> AnyResult<Option<Plan>> {
        let mut outputs = Vec::new();

        for entry in WalkDir::new(&config.reference_tracks_dir).into_iter() {
            match rx.try_recv() {
                Ok(Request::Cancel) | Err(TryRecvError::Disconnected) => {
                    return Ok(None);
                }
                Err(TryRecvError::Empty) => {
                    // nop
                }
            }

            let entry = entry?;
            let infile = entry.path();
            let outfiles: AnyResult<Vec<_>> = config.outputs_for(&infile).collect();
            let outfiles = outfiles?;

            // todo check if outfile already exists

            outputs.push(InfilePlan {
                infile: infile.to_owned(),
                outfiles,
            })
        }

        Ok(Some(Plan {
            outputs,
        }))
    }
}

pub mod exec {

    use super::plan::{Plan, InfilePlan};

    use rx::prelude::*;
    use rx::rayon::{self, prelude::*};

    use rx::walkdir::{self, WalkDir, DirEntry};
    use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::path::{PathBuf, Path};
    use std::fs;

    pub enum Request {
        Cancel,
    }

    pub enum Response {
        NextResult(ConvertResult),
        Done,
        Cancelled,
    }

    #[derive(Debug)]
    pub struct ConvertResult {
        pub in_path: PathBuf,
        pub out_path: PathBuf,
        pub format: Format,
        pub error: AnyResult<()>,
    }

    pub fn spawn(plan: Plan) -> (
        SyncSender<Request>,
        Receiver<Response>,
    ) {
        let (in_tx, in_rx) = sync_channel(1);
        let (out_tx, out_rx) = sync_channel(1);

        thread::spawn(move || {
            run(plan, in_rx, out_tx)
        });

        (in_tx, out_rx)
    }

    fn run(
        plan: Plan,
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

        plan.outputs.par_iter().for_each(|infile_plan| {
            convert_file(infile_plan, &tx, &cancel);
        });

        if !cancel.load(Ordering::SeqCst) {
            let _ = tx.send(Response::Done);
        } else {
            let _ = tx.send(Response::Cancelled);
        }
    }

    fn convert_file(
        plan: &InfilePlan,
        tx: &SyncSender<Response>,
        cancel: &AtomicBool,
    ) {
        let plan = FilePlan::new(
            plan,
            tx,
            cancel,
        );

        plan.run();
    }

    use rx::prelude::*;
    use rx::rand::Rng;
    use crate::types::{Format, SampleRate, BitDepth};
    use std::collections::BTreeMap;
    use crate::io::{PcmReader, PcmWriter, PanicPcmWriter, Buf, Props};
    use crate::samplerate::SampleRateConverter;
    use crate::bitdepth::BitDepthConverter;
    use crate::codecs;
    use super::OutFile;

    type FormatPlan =
        BTreeMap<
            SampleRate,
            BTreeMap<BitDepth, Vec<OutFile>>
        >;
    type ConverterPlan =
        BTreeMap<
            SampleRate, (
                SampleRateConverter,
                BTreeMap<
                    BitDepth, (
                        BitDepthConverter,
                        Vec<Option<OutFileWriter>>
                    )
                >
            )
        >;
    struct FilePlan<'up> {
        cancel: &'up AtomicBool,
        tx: &'up SyncSender<Response>,
        infile: &'up Path,
        sample_rates: FormatPlan,
    }

    struct OutFileWriter {
        path: PathBuf,
        tmp_path: PathBuf,
        format: Format,
        writer: Box<dyn PcmWriter>,
    }

    impl<'up> FilePlan<'up> {
        fn new<'up_>(
            plan: &'up_ InfilePlan,
            tx: &'up_ SyncSender<Response>,
            cancel: &'up_ AtomicBool,
        ) -> FilePlan<'up_> {
            let mut sample_rates: BTreeMap<SampleRate, BTreeMap<BitDepth, Vec<OutFile>>> = BTreeMap::new();

            for outfile in &plan.outfiles {
                let mut bit_depths = sample_rates.entry(outfile.format.sample_rate).or_default();
                let mut out_files = bit_depths.entry(outfile.format.bit_depth).or_default();
                out_files.push(outfile.clone());
            }

            FilePlan {
                cancel,
                tx,
                infile: &plan.infile,
                sample_rates,
            }
        }

        fn converter_plan(&self, source_props: &Props) -> ConverterPlan {
            self.sample_rates.iter().map(|args| {
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
                        let out_dir = outfile.path.parent();
                        // fixme only call create_dir_all once per directory
                        // fixme error handling
                        if let Some(out_dir) = out_dir {
                            match std::fs::create_dir_all(&out_dir) {
                                Ok(_) => { }
                                Err(e) => todo!("{e}"),
                            }
                        }
                        let tmp_path = tmp_path(&outfile.path);
                        Some(OutFileWriter {
                            path: outfile.path.clone(),
                            tmp_path: tmp_path.clone(),
                            format: outfile.format,
                            writer: codecs::writer(&tmp_path, Props {
                                channels: source_props.channels,
                                format: outfile.format,
                            }),
                        })
                    }).collect();

                    (
                        *bit_depth,
                        (
                            BitDepthConverter::new(
                                BitDepth::F32,
                                *bit_depth,
                                source_props.format.bit_depth,
                            ),
                            writers,
                        ),
                    )
                }).collect();

                (
                    *sample_rate,
                    (
                        SampleRateConverter::new(
                            source_props.format.sample_rate,
                            *sample_rate,
                            source_props.channels,
                        ),
                        bit_depths,
                    ),
                )
            }).collect()
        }

        fn prepare(&self) -> AnyResult<(
            Box<dyn PcmReader>,
            ConverterPlan,
            BitDepthConverter,
        )> {
            let mut reader = codecs::reader(&self.infile)?;
            let source_props = reader.props()?;
            let mut sample_rates = self.converter_plan(&source_props);
            let mut f32_converter = BitDepthConverter::new(
                source_props.format.bit_depth,
                BitDepth::F32,
                source_props.format.bit_depth,
            );

            Ok((reader, sample_rates, f32_converter))
        }
            

        fn run(&self) {
            let (
                mut reader,
                mut sample_rates,
                mut f32_converter,
            ) = match self.prepare() {
                Ok(preps) => preps,
                Err(e) => {
                    todo!();
                }
            };
            let mut buf = Buf::Uninit;
            let mut read_error = Ok(());

            loop {
                if self.cancel.load(Ordering::SeqCst) {
                    break;
                }

                match reader.read(&mut buf) {
                    Ok(()) => { },
                    Err(e) => {
                        read_error = Err(Arc::new(e));
                        break;
                    }
                }

                let buf = f32_converter.convert(&buf);

                // At this point `buf` either has data,
                // or is empty if EOF. Even if EOF
                // we may need to keep doing sample rate conversion
                // to pick up the any remaining buffers in the SRC.

                let keep_going = sample_rates.par_iter_mut().try_for_each(|args| {
                    let (
                        sample_rate,
                        (
                            sample_rate_converter,
                            bit_depths,
                        ),
                    ) = args;

                    let buf = if !buf.is_empty() {
                        let buf = sample_rate_converter.convert(&buf);
                        if buf.is_empty() {
                            // The SRC didn't produce any samples,
                            // which might happen with short input and
                            // reducing the sample rate.
                            return Some(());
                        } else {
                            buf
                        }
                    } else {
                        sample_rate_converter.finalize()
                    };

                    bit_depths.par_iter_mut().try_for_each(|args| {
                        let (
                            bit_depth,
                            (
                                bit_depth_converter,
                                writers,
                            ),
                        ) = args;

                        let buf = bit_depth_converter.convert(buf);

                        writers.par_iter_mut().try_for_each(|writer_ref| {

                            // todo: is it worth doing this check here?
                            // we already did it in the outer loop.
                            if self.cancel.load(Ordering::SeqCst) {
                                return None;
                            }

                            // If there is any error writing the file we will
                            // handle it now, and set the writer to `None` for
                            // future iterations.
                            let writer = std::mem::replace(writer_ref, None);

                            // If the writer is `None` then there was an error
                            // previously.
                            if let Some(mut writer) = writer {
                                let mut handle_error = |writer: OutFileWriter, e| {
                                    // Drop the writer so it closes any handles.
                                    // This might matter on windows.
                                    drop(writer.writer);
                                    let res = fs::remove_file(&writer.tmp_path);
                                    if let Err(e) = res {
                                        error!("error removing temp file while handling error");
                                    }
                                    self.tx.send(Response::NextResult(
                                        ConvertResult {
                                            in_path: self.infile.to_owned(),
                                            out_path: writer.path,
                                            format: writer.format,
                                            error: Err(e),
                                        }
                                    ));
                                };
                                if !buf.is_empty() {
                                    let res = writer.writer.write(buf);
                                    if let Err(e) = res {
                                        handle_error(writer, e);
                                    } else {
                                        *writer_ref = Some(writer);
                                    }
                                } else {
                                    let res = writer.writer.finalize();
                                    if let Err(e) = res {
                                        handle_error(writer, e);
                                    } else {
                                        let format = writer.format;
                                        // Drop the writer so it closes any handles.
                                        // This might matter on windows.
                                        drop(writer.writer);
                                        let res = fs::rename(&writer.tmp_path, &writer.path);
                                        if let Err(e) = res {
                                            writer.writer = Box::new(PanicPcmWriter);
                                            handle_error(writer, e.into());
                                        } else {
                                            // success!
                                            self.tx.send(Response::NextResult(
                                                ConvertResult {
                                                    in_path: self.infile.to_owned(),
                                                    out_path: writer.path,
                                                    format: writer.format,
                                                    error: Ok(()),
                                                }
                                            ));
                                        }
                                    }
                                }
                            }

                            Some(())
                        })
                    })
                });

                if keep_going.is_none() {
                    break;
                }

                if buf.is_empty() {
                    break;
                }
            }

            // Do cleanups and send cancellation / file read errors.
            for (_, (_, bit_depths)) in sample_rates.into_iter() {
                for (_, (_, writers)) in bit_depths.into_iter() {
                    for writer in writers {
                        match writer {
                            None => {
                                // The outfile has been completely handled,
                                // either written, or an error.
                            }
                            Some(writer) => {
                                // Conversion was cancelled or there was
                                // an error reading the infile.

                                // Drop the writer so it closes any handles.
                                // This might matter on windows.
                                drop(writer.writer);

                                let res = fs::remove_file(&writer.tmp_path);
                                if let Err(e) = res {
                                    error!("error removing temp file while handling error: {e}");
                                }

                                match read_error.as_ref() {
                                    Ok(()) => {
                                        self.tx.send(Response::NextResult(
                                            ConvertResult {
                                                in_path: self.infile.to_owned(),
                                                out_path: writer.path,
                                                format: writer.format,
                                                error: Err(anyhow!("cancelled")),
                                            }
                                        ));
                                    }
                                    Err(e) => {
                                        self.tx.send(Response::NextResult(
                                            ConvertResult {
                                                in_path: self.infile.to_owned(),
                                                out_path: writer.path,
                                                format: writer.format,
                                                // fixme: don't stringify this error
                                                error: Err(anyhow!("{}", e).context("file read error")),
                                            }
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn tmp_path(path: &Path) -> PathBuf {
        let mut tmp_path = path.to_owned();
        let ext = path.extension().expect("extension");
        let ext = ext.to_string_lossy().to_string();
        let random: u16 = rx::rand::thread_rng().gen();
        let ext = format!("{ext}.{random:04X}.tmp");
        tmp_path.set_extension(ext);
        tmp_path
    }
}

use crate::types::{Format, Codec};
use self::config::Config;
use std::path::{Path, PathBuf};
use rx::prelude::*;
use rx::tera::{Tera, Context as TeraContext};
use rx::serde::Serialize;

#[derive(Clone)]
pub struct OutFile {
    path: PathBuf,
    format: Format,
}

impl Config {
    fn outputs_for<'s>(&'s self, path: &'s Path) -> impl Iterator<Item = AnyResult<OutFile>> + 's {
        self.formats.iter().copied().map(|format| {
            Ok(OutFile {
                path: self.outfile_for(path, format)?,
                format,
            })
        })
    }

    fn outfile_for(&self, path: &Path, format: Format) -> AnyResult<PathBuf> {
        #[derive(Serialize)]
        struct OutPathVars {
            out_root_dir: PathBuf,
            relative_path: PathBuf,
            file_stem: String,
            format_ext: String,
        }

        let outpath_vars = OutPathVars {
            out_root_dir: self.out_root_dir.clone(),
            relative_path: path.strip_prefix(&self.reference_tracks_dir)?.to_owned(),
            file_stem: if let Some(file_stem) = path.file_stem() {
                file_stem.to_str().ok_or_else(|| {
                    anyhow!("can't convert file stem to UTF-8")
                })?.to_string()
            } else {
                bail!("no file stem")
            },
            format_ext: match format.codec {
                Codec::Wav => "wav",
                Codec::Flac => "flac",
                Codec::Vorbis => "ogg",
            }.to_string(),
        };

        let mut tera = Tera::default();
        tera.add_raw_template("template", &self.out_path_template)?;

        let context = TeraContext::from_serialize(&outpath_vars)?;
        let path = tera.render("template", &context)?;

        Ok(PathBuf::from(path))
    }
}
