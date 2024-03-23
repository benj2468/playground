{ clangStdenv, cmake, adk, makeWrapper, ... }:
clangStdenv.mkDerivation rec {
  name = "c-app";
  src = ./.;

  nativeBuildInputs = [ cmake makeWrapper ];
  buildInputs = [ adk ];

  postFixup = ''
    wrapProgram $out/bin/${name} \
      --set LD_LIBRARY_PATH ${adk}/lib \
      --set DYLD_LIBRARY_PATH ${adk}/lib
  '';
}
