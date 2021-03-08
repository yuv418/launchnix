#![recursion_limit = "256"]
mod image;
mod morph;
mod nix;
mod vm;
mod xml;

use std::fs::{canonicalize, read_to_string};
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    deployfile: PathBuf,
}

fn main() {
    let opt = Args::from_args();
    match canonicalize(opt.deployfile) {
        Ok(deployfile_unwrapped) => {
            let deployfile = deployfile_unwrapped.to_str().unwrap();

            let mut vm = vm::VM::from_nixfile(deployfile);
            vm.apply().unwrap_or_else(|err| {
                error_exit(&format!(
                    "Something went wrong when deploying your VM.\nDetailed information: {:#?}",
                    err
                ))
            });
        }
        Err(era) => error_exit(&format!(
            "The deployment file you provided doesn't exist, or we can't read it.\nDetailed information: {:#?}",
            era
        ))
    };
}

fn error_exit(msg: &str) {
    eprintln!("{}", msg);
    exit(1);
}
