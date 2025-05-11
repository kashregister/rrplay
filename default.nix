{
  pkgs ? import <nixpkgs> {},
  src ? ./.,
}: let
  theSource = src;
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = "${src}/Cargo.lock";
    src = pkgs.lib.cleanSource theSource;

    buildInputs = with pkgs; [
      dbus
      alsa-lib
    ];

    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
  }
