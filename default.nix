{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage {
  pname = "rrplay";
  version = "1.3.0";
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
