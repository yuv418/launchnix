use std::process::Command;
use std::env;
use std::fs;
use std::io::Error;
use ergo_fs::*;

pub fn build_image(ssh_pubkey: &str) -> Result<String, ExpandError> {
    // nix-build '<nixpkgs/nixos>' -A config.system.build.qcow2 -I nixos-config=./baseimage.nix

    let abspath_ssh_pubkey = expand(ssh_pubkey)?;

    let mut build = Command::new("nix-build")
        .arg("<nixpkgs/nixos>")
        .arg("-A")
        .arg("config.system.build.qcow2")
        .arg("--arg")
        .arg("configuration")
        .arg(&format!("{{ imports = [ (import ./nix/baseimage.nix \"{}\") ]; }}", abspath_ssh_pubkey))
        .spawn()
        .unwrap();

    let status = build.wait();

    let mut opath = env::current_dir().unwrap();
    opath.push("result");
    let mut opath = opath.read_link().unwrap();
    opath.push("nixos.qcow2");


    Ok(opath.to_str().unwrap().to_string())
}
