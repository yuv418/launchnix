use std::process::Command;

pub fn build_image() -> String {
    // nix-build '<nixpkgs/nixos>' -A config.system.build.qcow2 -I nixos-config=./baseimage.nix

    let mut build = Command::new("nix-build")
        .arg("<nixpkgs/nixos>")
        .arg("-A")
        .arg("config.system.build.qcow2")
        .arg("-I")
        .arg("nixos-config=./nix/baseimage.nix")
        .spawn()
        .unwrap();

    let status = build.wait();

    String::from("nix/result/nix.qcow2")
}
