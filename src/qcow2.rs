// Wrapper to qcow-img program

use std::path::PathBuf;
use std::process::Command;

pub fn create(output_path: &PathBuf, size: u64) -> Result<(), String> {
    // Size is in bytes

    let lazy_error_message = "Failed to create the blank image for your non-NixOS VM.";
    let img_handle = Command::new("qemu-img")
        .arg("create")
        .arg("-g")
        .arg("qcow2")
        .arg(output_path.to_str().unwrap())
        .arg(size.to_string() + "M")
        .status()
        .map_err(|e| lazy_error_message)?;

    if img_handle.success() {
        Ok(())
    } else {
        Err(lazy_error_message.to_owned())
    }
}

fn dd(input_path: &str, output_path: &str) -> Result<(), String> {
    unimplemented!()
}
