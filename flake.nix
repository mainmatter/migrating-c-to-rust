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

        # `cheadergen` isn't in here on purpose: it runs its own pinned nightly
        # through `rustup` (`rustup run`/`rustup which`), which this shell
        # doesn't provide. CI installs both with the host's `rustup` instead;
        # see the `verify-solutions` job.
        baseTools = with pkgs; [
          rustToolchain
          mdbook
          # Sets LIBCLANG_PATH so the bindgen exercise can find libclang.
          rustPlatform.bindgenHook
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
