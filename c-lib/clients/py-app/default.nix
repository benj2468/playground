{ pkgs, hellow, hellow-py, ... }:
pkgs.python3Packages.buildPythonPackage rec {
  name = "py-app";
  src = ./.;
  format = "pyproject";

  nativeBuildInputs = [
    pkgs.makeWrapper
  ] ++ (with pkgs.python3Packages; [ setuptools ]);
  propagatedBuildInputs = [ hellow-py ];

  postFixup = ''
    wrapProgram $out/bin/${name} \
      --set LD_LIBRARY_PATH ${hellow}/lib \
      --set DYLD_LIBRARY_PATH ${hellow}/lib
  '';
}
