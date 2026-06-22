{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        baseTools = with pkgs; [
          mdbook
        ];
        authorTools = with pkgs; [
          dprint
          typos
          clang-tools
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
