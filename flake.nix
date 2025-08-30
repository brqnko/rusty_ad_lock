{
  description = "database advisory lock implementation for rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        stdenv = pkgs.stdenv;
      in
      {
        formatter = pkgs.nixpkgs-fmt;
        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              pkg-config
              clang
            ];
            packages = with pkgs; [
            ];
            shellHook = ''
            '';
          };
          shellHook = ''
          '';
        };
      }
    );
}
