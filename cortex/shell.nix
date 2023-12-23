{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    cargo-expand
    cargo-generate
    llvmPackages.bintools
    iconv
    python3
    maturin
    gprof2dot
    graphviz
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
