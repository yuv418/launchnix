{
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
    rustChannel = "1.50.0";
    packageFun = import ./Cargo.nix;
    packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [
      (pkgs.rustBuilder.rustLib.makeOverride {
          name = "launchnix";
          overrideAttrs = drv: {
            propagatedNativeBuildInputs = drv.propagatedNativeBuildInputs or [ ] ++ [
                pkgs.libvirt
            ];
            installPhase = drv.installPhase + ''
                cp -r $src/nix $out/bin/
            '';
          };
      })
    ];
    localPatterns = [ ''^(src|nix)(/.*)?'' ''[^/]*\.(rs|toml)$'' ];
  };
in {
    launchnix = (rustPkgs.workspace.launchnix {});
}
