{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "rrplay";
  version = "1.2.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;

  buildInputs = with pkgs; [
    dbus
    alsa-lib
    pkg-config
  ];

  nativeBuildInputs = with pkgs; [
    dbus
    alsa-lib
    pkg-config
  ];
}
