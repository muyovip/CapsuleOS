{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  name = "capsuleos-dev";

  buildInputs = with pkgs; [
    rustc
    cargo
    limine
    xorriso
    qemu
  ];

  shellHook = ''
    echo "cos(Blue) = 1 — CapsuleOS Dev Shell Activated"
    export PS1="\n⊙₀ \[\e[34m\]CapsuleOS\[\e[0m\] > "
  '';
}