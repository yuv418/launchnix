use std::env;
use std::fs;
use std::io::Error;
use std::process::Command;
use std::io;
use ergo_fs::*;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn exec_morph(ip: &str, ssh_pubkey: &str, deployment_file_path: &str) -> Result<(), Box<std::error::Error + 'static>> {
    // file_path goes to deploymentPath
    // ip goes to domIP
    // hwConfigPath gets executable path + /nix/hwconfig.nix

    let mut exe_path = env::current_exe()?;
    exe_path.pop();
    println!("{:?}", exe_path);

    let mut hwconfig_path = exe_path.clone();
    let mut tomorph_path = exe_path;

    hwconfig_path.push("nix/hwconfig.nix");
    tomorph_path.push("nix/tomorph.nix");

    // Morph doesn't let you pass arguments, so manually replace string values (*sigh).

    let deployment_abspath = fs::canonicalize(deployment_file_path)?;
    let deployment_abspath = deployment_abspath.to_str().unwrap();

    let sshpubkey_abspath = expand(ssh_pubkey)?;

    let tomorph_string = fs::read_to_string(tomorph_path)?
        .replace("deploymentPath", deployment_abspath)
        .replace("domIP", ip)
        .replace("hwConfigPath", hwconfig_path.to_str().unwrap())
        .replace("sshPubKeyPath", &sshpubkey_abspath);

    println!("{}", tomorph_string);

    let mut temp_nix = env::temp_dir();
    let temp_nix_filename: String = thread_rng() // From https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    temp_nix.push(temp_nix_filename);
    fs::write(&temp_nix, tomorph_string)?;


    let mut build = Command::new("morph")
        .arg("deploy")
        .arg("--upload-secrets")
        .arg(temp_nix.to_str().unwrap())
        .arg("switch")
        .env("SSH_USER", "root") // VMs do this by default*/
        .spawn()
        .unwrap();

    build.wait().expect("failed");

    fs::remove_file(temp_nix)?;

    Ok(())

}
