{ pkgs, hellow, ... }:
pkgs.buildGoModule rec {
  name = "goApp";
  version = "0.1.0";
  src = ./.;

  nativeBuildInputs = [ pkgs.makeWrapper ];
  buildInputs = [ hellow ];
  vendorHash = null;

  postFixup = ''
    wrapProgram $out/bin/${name} \
      --set LD_LIBRARY_PATH ${hellow}/lib \
      --set DYLD_LIBRARY_PATH ${hellow}/lib
  '';
}
