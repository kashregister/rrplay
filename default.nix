{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "rrplay";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;

  buildInputs = with pkgs; [
    dbus
    alsa-lib
  ];

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
}
