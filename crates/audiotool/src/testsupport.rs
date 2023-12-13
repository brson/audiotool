use rx::prelude::*;
use rx::rand_pcg::Pcg64Mcg;
use rx::rand::Rng;
use rx::itertools::Itertools;
use std::path::Path;
use std::iter;
use crate::convert as cvt;
use crate::types::*;
use crate::io::{Props, Buf};
use crate::codecs;

pub fn write_test_file(
    path: &Path,
    props: Props,
    frames: u32,
) -> AnyResult<Buf> {
    let mut rng = Pcg64Mcg::new(0);
    let samples = frames as usize * props.channels as usize;
    let buf = match props.format.bit_depth {
        BitDepth::F32 => {
            Buf::F32(
                iter::from_fn(|| {
                    Some(rng.gen_range(-1.0..=1.0))
                }).take(samples).collect()
            )
        }
        _ => todo!(),
    };

    let mut writer = codecs::writer(path, props);
    writer.write(&buf)?;
    writer.finalize()?;

    Ok(buf)
}

pub fn read_file(path: &Path) -> AnyResult<(Props, Buf)> {
    #[extension_trait]
    impl BufExt for Buf {
        fn append(&mut self, other: &Buf) {
            match (self, other) {
                (this @ Buf::Uninit, Buf::F32(other)) => {
                    *this = Buf::F32(other.clone());
                }
                (Buf::F32(ref mut this), Buf::F32(other)) => {
                    this.extend(other.iter());
                },
                 _ => todo!(),
            }
        }
    }

    let mut reader = codecs::reader(path)?;
    let mut all_buf = Buf::Uninit;
    let mut tmp_buf = Buf::Uninit;

    loop {
        reader.read(&mut tmp_buf)?;

        if tmp_buf.is_empty() {
            break;
        }

        all_buf.append(&tmp_buf);
    }

    Ok((reader.props()?, all_buf))
}

pub fn run_convert(config: cvt::config::Config) -> AnyResult<()> {
    let (_tx, rx) = cvt::plan::spawn(config);

    let plan = match rx.recv().expect("recv") {
        cvt::plan::Response::Done(Ok(Some(plan))) => plan,
        cvt::plan::Response::Done(Ok(None)) => panic!(),
        cvt::plan::Response::Done(Err(e)) => panic!("{e}"),
    };

    let (_tx, rx) = cvt::exec::spawn(plan);

    loop {
        let resp = rx.recv()?;

        match resp {
            cvt::exec::Response::NextResult(_res) => {
                //println!("{res:#?}");
            }
            cvt::exec::Response::Done => {
                break;
            }
            cvt::exec::Response::Cancelled => {
                panic!();
            }
        }
    }

    Ok(())
}
