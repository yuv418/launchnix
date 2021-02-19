use std::process::Command;

use serde::{Deserialize, de::DeserializeOwned};
use serde_json;


// TODO make this a templated function
pub fn from_nixfile<T>(file_path: &str) -> T
    where T: DeserializeOwned {
    let op_json_exec = Command::new("nix")
        .arg("eval")
        .arg("-f")
        .arg(file_path)
        .arg("deployment")
        .arg("--json")
        .output()
        .expect("Launchnix failed to read your deployment file correctly. Check your file for syntax errors/see if it has the 'deployment' attribute map defined.");

    // let op_json = String::from_utf8_lossy(&op_json_exec.stdout);
    serde_json::from_slice(&op_json_exec.stdout).unwrap()
}
