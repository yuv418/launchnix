args@{
  system ? builtins.currentSystem,
  nixpkgsMozilla ? builtins.fetchGit {
    url = https://github.com/mozilla/nixpkgs-mozilla;
    rev = "18cd4300e9bf61c7b8b372f07af827f6ddc835bb";
  },
  cargo2nix ? builtins.fetchGit {
    url = https://github.com/cargo2nix/cargo2nix;
    ref = "master";
  },
}:

let
  rustOverlay = import "${nixpkgsMozilla}/rust-overlay.nix";
  cargo2nixOverlay = import "${cargo2nix}/overlay";

  pkgs = import <nixpkgs> {
    inherit system;
    overlays = [ cargo2nixOverlay rustOverlay ];
  };

  rustPkgs = pkgs.rustBuilder.makePackageSet' {
    rustChannel = "stable";
    packageFun = import ./Cargo.nix;
    packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [
      (pkgs.rustBuilder.rustLib.makeOverride {
          name = "launchnix";
          overrideAttrs = drv: {
            buildInputs = drv.buildInputs ++ [ pkgs.makeWrapper ];
            propagatedNativeBuildInputs = drv.propagatedNativeBuildInputs or [ ] ++ [
                pkgs.libvirt
            ];
            installPhase = drv.installPhase + ''
                mv $bin/bin/launchnix $bin/bin/launchnix-bin
                makeWrapper $bin/bin/launchnix-bin $bin/bin/launchnix --set PATH ${ pkgs.lib.makeBinPath [ pkgs.morph pkgs.nix pkgs.gnutar pkgs.gzip pkgs.git pkgs.openssh ] }

                cp -r $src/nix $bin/bin/
            '';
          };
      })
    ];
    localPatterns = [ ''^(src|nix)(/.*)?'' ''[^/]*\.(rs|toml)$'' ];
  };
in {
    launchnix = rustPkgs.workspace.launchnix {};
}
