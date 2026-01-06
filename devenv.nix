{ pkgs, ... }:

{
  packages = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
  ];

  languages.rust.enable = true;
}
