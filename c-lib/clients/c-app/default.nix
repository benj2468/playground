{ pkgs, hellow, ...}: 
pkgs.clangStdenv.mkDerivation {
  name = "hellow-c";
  src = ./.;

  nativeBuildInputs = [ pkgs.cmake pkgs.makeWrapper ];
  buildInputs = [ hellow ];
  postFixup = ''
    wrapProgram $out/bin/hellow-c \
      --set LD_LIBRARY_PATH ${hellow}/lib \
      --set DYLD_LIBRARY_PATH ${hellow}/lib
  '';
}