{
  description = "A Nix-flake-based Rust development environment";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    ...
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import nixpkgs {
            inherit system;
            overlays = [rust-overlay.overlays.default self.overlays.default];
          };
        });
  in {
    overlays.default = final: prev: {
      rustToolchain = let
        rust = prev.rust-bin;
      in
        if builtins.pathExists ./rust-toolchain.toml
        then rust.fromRustupToolchainFile ./rust-toolchain.toml
        else if builtins.pathExists ./rust-toolchain
        then rust.fromRustupToolchainFile ./rust-toolchain
        else
          rust.stable.latest.default.override {
            extensions = ["rust-src" "rustfmt"];
          };
    };

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        buildInputs = [
          pkgs.openssl
          pkgs.pkg-config
          pkgs.cargo-deny
          pkgs.cargo-edit
          pkgs.cargo-watch
          pkgs.rust-analyzer
          pkgs.alsa-lib.dev
        ];

        shellHook = ''
          export RUST_SRC_PATH="${pkgs.rustToolchain}/lib/rustlib/src/rust/library"
        '';
      };
    });

    packages = forEachSupportedSystem ({pkgs}: {
      default = pkgs.rustPlatform.buildRustPackage rec {
        pname = "rrplay";
        version = "0.1.0";
        src = "./";
        cargoLock.lockFile = ./Cargo.lock;
        buildInputs = [pkgs.dbus pkgs.alsa-lib.dev];
        nativeBuildInputs = [pkgs.pkg-config];
      };
    });
  };
}
