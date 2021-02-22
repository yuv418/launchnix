#![recursion_limit="256"] mod xml;
mod nix;
mod vm;
mod image;
mod morph;

use std::path::PathBuf;
use std::fs::canonicalize;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    deployfile: PathBuf,
}

fn main() {

    let opt = Args::from_args();
    let deployfile = canonicalize(opt.deployfile).expect("The deployment file you provided doesn't exist.");
    let deployfile = deployfile.to_str().unwrap();

    let mut vm = vm::VM::from_nixfile(deployfile);
    vm.apply();
}
