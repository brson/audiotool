use rx::prelude::*;
use rx::clap::{self, Parser as _};
use std::path::PathBuf;
use std::fs;
use std::thread;

mod convert;
mod split;
mod ctrlc;

fn main() -> AnyResult<()> {
    rx::extras::init();
    ctrlc::init();

    let cli = Cli::parse();
    cli.run()?;

    Ok(())
}

#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
    #[command(flatten)]
    args: Args,
}

#[derive(clap::Subcommand)]
enum Command {
    Convert(ConvertCommand),
    Template(TemplateCommand),
}

#[derive(clap::Args)]
struct Args {
}

#[derive(clap::Args)]
struct ConvertCommand {
    config: PathBuf,
}

#[derive(clap::Args)]
struct TemplateCommand {
    config: PathBuf,
}

impl Cli {
    fn run(&self) -> AnyResult<()> {
        match &self.cmd {
            Command::Convert(cmd) => cmd.run(&self.args),
            Command::Template(cmd) => cmd.run(&self.args),
        }
    }
}

impl ConvertCommand {
    fn run(&self, _args: &Args) -> AnyResult<()> {
        use audiotool::convert as cvt;

        let config = fs::read_to_string(&self.config)?;
        let config: cvt::config::Config = rx::toml::from_str(&config)?;

        let (tx, rx) = cvt::plan::spawn(config);

        thread::spawn(move || {
            ctrlc::wait();
            let _ = tx.send(cvt::plan::Request::Cancel);
        });

        let plan = match rx.recv().expect("recv") {
            cvt::plan::Response::Done(Ok(Some(plan))) => plan,
            cvt::plan::Response::Done(Ok(None)) => {
                // cancelled
                return Ok(());
            }
            cvt::plan::Response::Done(Err(e)) => {
                return Err(e);
            }
        };

        let (tx, rx) = cvt::exec::spawn(plan);

        thread::spawn(move || {
            ctrlc::wait();
            let _ = tx.send(cvt::exec::Request::Cancel);
        });

        loop {
            let resp = rx.recv()?;

            match resp {
                cvt::exec::Response::NextResult(res) => {
                    println!("{res:#?}");
                }
                cvt::exec::Response::Done => {
                    break;
                }
                cvt::exec::Response::Cancelled => {
                    break;
                }
            }
        }

        Ok(())
    }
}

impl TemplateCommand {
    fn run(&self, _args: &Args) -> AnyResult<()> {
        use audiotool::convert as cvt;

        let config = cvt::config::Config::template();
        let config = rx::toml::to_string(&config)?;

        fs::write(&self.config, &config)?;

        Ok(())
    }
}
