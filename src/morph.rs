use ergo_fs::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;
use std::env;
use std::fs;
use std::process::Command;
use tera::Context;
use tera::Tera;

#[derive(Serialize)]
struct MorphContext<'a> {
    dom_ip: &'a str,
    sshpubkey_abspath: &'a str,
    deployment_abspath: &'a str,
    hwconfig_path: &'a str,
    static_ips: &'a Option<Vec<String>>,
}

pub fn exec_morph(
    ip: &str,
    ssh_pubkey: &str,
    deployment_file_path: &str,
    static_ips: &Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
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

    let tomorph_str = fs::read_to_string(tomorph_path)?;
    let tomorph_str = Tera::one_off(
        &tomorph_str,
        &Context::from_serialize(MorphContext {
            dom_ip: ip,
            static_ips,
            deployment_abspath,
            sshpubkey_abspath: &sshpubkey_abspath,
            hwconfig_path: hwconfig_path.to_str().unwrap(),
        })?,
        false,
    )?;
    println!("{}", tomorph_str);
    // std::process::exit(0);

    let mut temp_nix = env::temp_dir();
    let temp_nix_filename: String = thread_rng() // From https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    temp_nix.push(temp_nix_filename);
    fs::write(&temp_nix, tomorph_str)?;

    let mut build = Command::new("morph")
        .arg("deploy")
        .arg("--upload-secrets")
        .arg(temp_nix.to_str().unwrap())
        .arg("switch")
        .env("SSH_USER", "root") // VMs do this by default*/
        .env("SSH_SKIP_HOST_KEY_CHECK", "1") // VMs do this by default*/
        .spawn()
        .unwrap();

    build.wait().expect("failed");

    fs::remove_file(temp_nix)?;

    Ok(())
}
