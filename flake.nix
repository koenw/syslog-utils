{
  description = "Syslog utilities for troubleshooting & testing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.stable.latest.default;
          rustc = pkgs.rust-bin.stable.latest.default;
        };

        rustPackage = rustPlatform.buildRustPackage {
          name = "syslog-client";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; [
            gcc
            openssl
          ];
        };

        syslog-client = rustPlatform.buildRustPackage {
          name = "syslog-client";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; [
            gcc
            openssl
          ];
        };

        syslog-server = rustPlatform.buildRustPackage {
          name = "syslog-server";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; [
            gcc
            openssl
          ];
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = "syslog-utils";
          config = {
            Cmd = [ "${syslog-server}/bin/syslog-server" ];
          };
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            rust-bin.stable.latest.default
            gcc
            pkg-config
            openssl
            just
          ];
          shellHook = ''
            user_shell=$(getent passwd "$(whoami)" |cut -d: -f 7)

            just --color=always -l |awk '/^Available recipes:/ gsub(/Available rescipes:/, "The following `just` commands are available:")'

            exec "$user_shell"

          '';
        };

        packages = {
          syslog-client = syslog-client;
          syslog-server = syslog-server;
          dockerImage = dockerImage;
          default = syslog-client;
        };
      }
    );
}
