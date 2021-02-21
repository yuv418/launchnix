# This file and baseimage.nix copied from https://gist.github.com/tarnacious/f9674436fff0efeb4bb6585c79a3b9ff

{ pkgs, lib, ... }:

with lib;

{
  imports = [
    <nixpkgs/nixos/modules/profiles/qemu-guest.nix>
  ];

  config = {
    fileSystems."/" = {
      device = "/dev/disk/by-label/nixos";
      fsType = "ext4";
      autoResize = true;
    };

    boot.growPartition = true;
    boot.kernelParams = [ ];
    boot.loader.grub.device = "/dev/vda";
    boot.loader.timeout = 0;

    services.openssh.enable = true;
    services.openssh.permitRootLogin = "prohibit-password";

  };
}
