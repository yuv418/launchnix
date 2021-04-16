// Wrapper to qcow-img program

use std::path::PathBuf;
use std::process::Command;

pub fn create(output_path: &PathBuf, size: u64) -> Result<(), String> {
    // Size is in megabytes

    let lazy_error_message = "Failed to create the blank image for your non-NixOS VM.";
    let img_handle = Command::new("qemu-img")
        .arg("create")
        .arg("-f")
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

pub fn dd(input_path: &PathBuf, output_path: &PathBuf) -> Result<(), String> {
    let lazy_error_message = "Failed to load your image from backup.";

    let img_handle = Command::new("qemu-img")
        .arg("dd")
        .arg("-O")
        .arg("qcow2")
        .arg("if=".to_owned() + input_path.to_str().unwrap())
        .arg("of=".to_owned() + output_path.to_str().unwrap())
        .arg("bs=256M")
        .status()
        .map_err(|e| lazy_error_message)?;

    if img_handle.success() {
        Ok(())
    } else {
        Err(lazy_error_message.to_owned())
    }

}
