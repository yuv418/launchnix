// Wrapper to qcow-img program

use std::path::PathBuf;
use std::process::Command;

pub fn create(output_path: &PathBuf, size: u64) -> Result<(), String> {
    // Size is in bytes
    let img_handle = Command::new("qemu-img")
        .arg("create")
        .arg("-f")
        .arg("qcow2")
        .arg(output_path.to_str().unwrap())
        .arg(size.to_string() + "M")
        .status()
        .map_err(|e| "Failed to create the blank image for your non-NixOS VM.")?;

    Ok(())
}

fn dd(input_path: &str, output_path: &str) -> Result<(), String> {
    unimplemented!()
}
