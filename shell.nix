{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/refs/tags/25.05.tar.gz") {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
  	bun
  	rustc
  	cargo
  	wasm-pack
  	lld
  ];
  packages = with pkgs; [
  ];
}
#
# let
#   myAppEnv = pkgs.poetry2nix.mkPoetryEnv {
#     projectDir = ./.;
#     editablePackageSources = {
#       my-app = ./src;
#     };
#   };
# in myAppEnv.env.overrideAttrs (oldAttrs: {
#   buildInputs = [
#     pkgs.rustc
#     pkgs.cargo
#   ];
#   packages = [
#     pkgs.python310Packages.pytest
#     pkgs.black
#     pkgs.isort
#   ];
# })
