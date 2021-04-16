use crate::pathutils::merge_exe_path;
use ergo_fs::expand;
use serde::Serialize;
use std::env;
use std::fs;
use std::io::Error;
use std::process::Command;
use tempfile::NamedTempFile;
use tera::{Context, Tera};

#[derive(Serialize)]
struct ImageContext<'a> {
    disk_size: u64,
    hwconfig_path: &'a str,
}

pub fn build_image(ssh_pubkey: &str, disk_size: u64) -> Result<String, Box<std::error::Error>> {
    // nix-build '<nixpkgs/nixos>' -A config.system.build.qcow2 -I nixos-config=./baseimage.nix

    let abspath_ssh_pubkey = expand(ssh_pubkey)?;
    let mut baseimage_abspath = env::current_exe().unwrap();
    baseimage_abspath.pop();
    baseimage_abspath.push("nix/baseimage.nix");

    // We need to modify this to include the default image size.

    let image_rendered = Tera::one_off(
        &fs::read_to_string(baseimage_abspath)?,
        &Context::from_serialize(ImageContext {
            disk_size,
            hwconfig_path: merge_exe_path("nix/hwconfig.nix").to_str().unwrap(),
        })?,
        false,
    )?;

    println!("Image rendered is \n:{}", image_rendered);
    let temp_imagef = NamedTempFile::new()?;
    fs::write(&temp_imagef, image_rendered)?;

    let mut build = Command::new("nix-build")
        .arg("<nixpkgs/nixos>")
        .arg("-A")
        .arg("config.system.build.qcow2")
        .arg("--arg")
        .arg("configuration")
        .arg(&format!(
            "{{ imports = [ (import {} \"{}\") ]; }}",
            temp_imagef.path().to_str().unwrap(),
            abspath_ssh_pubkey
        )) // TODO replace baseimage with an absolute path
        .spawn()?;

    let status = build.wait();

    let mut opath = env::current_dir().unwrap();
    opath.push("result");
    let mut opath = opath.read_link().unwrap();
    opath.push("nixos.qcow2");

    Ok(opath.to_str().unwrap().to_string())
}
