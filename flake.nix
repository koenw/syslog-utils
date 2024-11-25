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

        naerskDev = naersk.lib.${system}.override {
          cargo = toolchainFull;
          rustc = toolchainFull;
          clippy = toolchainFull;
        };

        staticPackage = naerskStatic.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [
            pkgsStatic.stdenv.cc
            pkgsStatic.openssl
          ];
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          # Tell Cargo to enable static compilation.
          # (https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags)
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        };

        nativePackage = naersk'.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [
            stdenv.cc
            openssl
          ];
        };

        devPackage = naerskDev.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [
            stdenv.cc
            openssl
          ];
        };

        # With explicit binary name for `nix run` (all packages contain both
        # binaries)
        client = naersk'.buildPackage {
          src = ./.;
          name = "syslog-client";
          nativeBuildInputs = with pkgs; [
            pkgsStatic.stdenv.cc
            pkgsStatic.openssl
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
          ];
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = "syslog-utils";
          tag = "latest";
          config = {
            Env = [
              "PATH=${staticPackage}/bin"
            ];
          };
        };
      in rec {
        packages = {
          # Statically linked
          static = staticPackage;
          # Dynamically linked
          native = nativePackage;

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
