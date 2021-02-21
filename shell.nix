with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "env";
  buildInputs = [
    cargo
    libvirt
  ];
}
