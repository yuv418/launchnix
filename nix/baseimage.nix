# Copied from https://gist.github.com/tarnacious/f9674436fff0efeb4bb6585c79a3b9ff

{ config, lib, pkgs, ... }:

with lib;

{
  imports =
    [
      ./hwconfig.nix
    ];

  system.build.qcow2 = import <nixpkgs/nixos/lib/make-disk-image.nix> {
    inherit lib config;

    pkgs = import <nixpkgs> { inherit (pkgs) system; }; # ensure we use the regular qemu-kvm package
    diskSize = 8192;
    format = "qcow2";
    configFile = pkgs.writeText "configuration.nix"
      ''
        {
          imports = [ ./hwconfig.nix ];
        }
      '';

  };
}
