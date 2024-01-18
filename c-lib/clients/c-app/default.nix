{ pkgs, hellow, ...}: 
pkgs.clangStdenv.mkDerivation {
  name = "hellow-c";
  src = ./.;

  nativeBuildInputs = [ pkgs.cmake ];
  buildInputs = [
    hellow
  ];
}