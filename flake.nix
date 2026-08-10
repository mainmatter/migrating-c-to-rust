{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = with pkgs; rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        baseTools = with pkgs; [
          rustToolchain
          mdbook
        ];
        authorTools = with pkgs; [
          dprint
          typos
          clang-tools
          rustfmt
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = baseTools;
        };
        devShells.author = pkgs.mkShell {
          buildInputs = baseTools ++ authorTools;
        };
      }
    );
}
