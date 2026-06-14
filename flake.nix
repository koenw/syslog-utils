{
  description = "Syslog utilities for troubleshooting & testing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Toolchain for use in development shell
        toolchainFull = with fenix.packages.${system}; combine [
          complete.rustc
          complete.cargo
          complete.clippy
          complete.rustfmt
          targets.x86_64-unknown-linux-musl.latest.rust-std
        ];

        # Toolchain to build static binaries
        toolchainStatic = with fenix.packages.${system}; combine [
          minimal.rustc
          minimal.cargo
          targets.x86_64-unknown-linux-musl.latest.rust-std
        ];

        naersk' = pkgs.callPackage naersk {};

        naerskStatic = naersk.lib.${system}.override {
          cargo = toolchainStatic;
          rustc = toolchainStatic;
        };

        staticPackage = naerskStatic.buildPackage {
          src = ./.;

          nativeBuildInputs = with pkgs; [
            perl
            pkg-config
          ];
          buildInputs = with pkgs; [
            pkgsStatic.openssl
          ];

          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          CC_x86_64-unknown-linux-musl = with pkgs.pkgsStatic.stdenv; "${cc}/bin/${cc.targetPrefix}gcc";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = with pkgs.pkgsStatic.stdenv; "${cc}/bin/${cc.targetPrefix}gcc";
        };

        syslog-utils = naersk'.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [
            stdenv.cc
            openssl
            perl
          ];
          buildInputs = with pkgs; [ pkg-config ];
        };

        # With explicit binary name for `nix run` (all packages contain both
        # binaries)
        client = naersk'.buildPackage {
          src = ./.;
          name = "syslog-client";
          nativeBuildInputs = with pkgs; [
            pkgsStatic.stdenv.cc
            pkgsStatic.openssl
            perl
          ];
        };

        # With explicit binary name for `nix run` (all packages contain both
        # binaries)
        server = naersk'.buildPackage {
          src = ./.;
          name = "syslog-server";
          nativeBuildInputs = with pkgs; [
            pkgsStatic.stdenv.cc
            pkgsStatic.openssl
            perl
          ];
        };

        naerskDev = naersk.lib.${system}.override {
          cargo = toolchainFull;
          rustc = toolchainFull;
          clippy = toolchainFull;
        };

        devPackage = naerskDev.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [
            stdenv.cc
            openssl
            perl
          ];
          buildInputs = with pkgs; [ pkg-config ];
        };


        check-all = naerskDev.buildPackage {
          src = ./.;
          name = "check-all";
          nativeBuildInputs = with pkgs; [
            pkgsStatic.stdenv.cc
            pkgsStatic.openssl
            perl
          ];
          fixupPhase = ''
            # Make sure we fail on all warnings, including Clippy lints
            export RUSTFLAGS="-Dwarnings"
            cargo fmt --check
            cargo clippy --no-deps
            cargo test
          '';
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = "syslog-utils";
          tag = "latest";
          config = {
            Env = [
              "PATH=${syslog-utils}/bin"
            ];
          };
        };
      in rec {
        checks = {
          default = check-all;
        };

        packages = {
          syslog-utils = syslog-utils;

          # Statically linked
          static = staticPackage;

          # For `nix run '.#client'`
          client = client;
          syslog-client = client;

          # For `nix run '.#server'`
          server = server;
          syslog-server = server;

          # For `nix run` or `nix run .`
          default = client;

          # The docker image contains both binaries
          docker = dockerImage;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = with packages; [ devPackage ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];

          buildInputs = with pkgs; [
            just
          ];

          shellHook = ''
            user_shell=$(getent passwd "$(whoami)" |cut -d: -f 7)

            just --color=always -l |awk '/^Available recipes:/ gsub(/^Available recipes:/, "The Following `just` commands are available:")'

            exec "$user_shell"
          '';
        };
      }
    );
}
