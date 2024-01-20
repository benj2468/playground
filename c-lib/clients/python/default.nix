{ pkgs, hellow, ... }:
let
  py-hello = pkgs.runCommand "hellow-py"
    {
      nativeBuildInputs = [ pkgs.gcc pkgs.python3 ];
      buildInputs = [ hellow ];
    } ''
    mkdir $out

    python -m venv .venv
    source .venv/bin/activate
    pip install ctypesgen
  
    ctypesgen -lhellow ${hellow}/include/*.h -o $out/hellow.py
  '';
in
pkgs.python3Packages.buildPythonPackage rec {
  name = "hellow-py";
  src = ./.;

  nativeBuildInputs = [
    pkgs.makeWrapper
    (pkgs.python3.withPackages (p: [ p.setuptools ]))
  ];

  format = "pyproject";

  postFixup = ''
    makeWrapper $out/bin/py-app $out/bin/${name} \
      --set LD_LIBRARY_PATH ${hellow}/lib \
      --set DYLD_LIBRARY_PATH ${hellow}/lib \
      --set PYTHONPATH ${py-hello}:$PYTHONPATH
  '';
}
