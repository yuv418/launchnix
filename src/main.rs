#![recursion_limit = "256"]
mod image;
mod morph;
mod nix;
mod vm;
mod xml;

use std::path::PathBuf;
use std::process::exit;
use std::{fmt::Display, fs::canonicalize};
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    #[derive(Debug, StructOpt)]
    enum Action {
        Deploy,
        Reboot,
        Shutdown,
        Start,
        SSH,
    }
}

impl Action {
    fn action_str(&self) -> &str {
        // Conflicts with something that arg_enum! does, so we need this.
        match self {
            Action::Deploy => "deploy",
            Action::Reboot => "reboot",
            Action::Shutdown => "shutdown",
            Action::SSH => "ssh into",
            Action::Start => "start",
        }
    }
}

#[derive(Debug, StructOpt)]
struct Args {
    action: Action,
    #[structopt(parse(from_os_str))]
    deployfile: PathBuf,
}

fn main() {
    let opt = Args::from_args();
    match canonicalize(&opt.deployfile) {
        Ok(deployfile_unwrapped) => {
            let deployfile = deployfile_unwrapped.to_str().unwrap();

            let mut vm = vm::VM::from_nixfile(deployfile);
            let get_dom = || -> virt::domain::Domain {
                match vm.dom(&vm.conn()) {
                    Ok(dom) => dom,
                    Err(_) => {
                        error_exit(
                            &format!("You can't {} a deployment without deploying it first! Try `launchnix deploy {} first`", &opt.action.action_str(), deployfile_unwrapped.to_str().unwrap()),
                        );
                        panic!() // Hack, because this will never run.
                    }
                }
            };
            match opt.action {
                Action::Deploy => vm.apply().unwrap_or_else(|err| {
                        error_exit(&format!(
                            "Something went wrong when deploying your VM.\nDetailed information: {:#?}",
                            err
                        ))
                    }),
                Action::Start => {
                    println!("Starting VM...");

                    let dom = get_dom(); // TODO make this logic less repetitive
                    if dom.is_active().unwrap() { // Ideally this would be in vm.rs later on
                        error_exit(&format!("The VM you wanted to start is already running."))
                    }

                    vm.dom_ip(&dom);
                }
                Action::Reboot => {
                    println!("Rebooting VM...");

                    vm.reboot(&get_dom());
                },
                Action::Shutdown => {
                    println!("Shutting down VM...");

                    let dom = get_dom();
                    if !dom.is_active().unwrap() { // Ideally this would be in vm.rs later on
                        error_exit(&format!("The VM you wanted to stop is already off."))
                    }
                    vm.shutdown(&dom);
                },
                Action::SSH => {
                    println!("SSHing into VM...");

                    vm.ssh();
                }
            }
        }
        Err(err) => error_exit(&format!(
            "The deployment file you provided doesn't exist, or we can't read it.\nDetailed information: {:#?}",
            err
        ))
    };
}

fn error_exit(msg: &str) {
    eprintln!("{}", msg);
    exit(1);
}
