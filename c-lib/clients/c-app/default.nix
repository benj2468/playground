{ pkgs, hellow, ...}: 
pkgs.clangStdenv.mkDerivation rec {
  name = "c-app";
  src = ./.;

  nativeBuildInputs = [ pkgs.cmake pkgs.makeWrapper ];
  buildInputs = [ hellow ];
  postFixup = ''
    wrapProgram $out/bin/${name} \
      --set LD_LIBRARY_PATH ${hellow}/lib \
      --set DYLD_LIBRARY_PATH ${hellow}/lib
  '';
}