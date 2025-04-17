{
  pkgs ? import <nixpkgs> {},
  src ? ./.,
}: let
  theSource = src;
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = "rrplay";
    version = "0.1.0";
    src = pkgs.lib.cleanSource theSource;
    cargoLock.lockFile = "${src}/Cargo.lock";
    buildInputs = with pkgs; [
      dbus
      alsa-lib
    ];
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    dbus = pkgs.dbus;
  }
