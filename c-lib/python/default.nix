 { pkgs, lib, hellow, ... }:
 pkgs.python3Packages.buildPythonPackage {
  name = "hellow";
  src = ./.;
  format = "pyproject";
  nativeBuildInputs = [ pkgs.gcc ];
  buildInputs = with pkgs.python3Packages; [ setuptools ];
  propagatedBuildInputs = [ hellow ];
  preBuild = ''
    PROJECT_DIR=src/hellow
    mkdir -p $PROJECT_DIR

    python -m venv .venv
    source .venv/bin/activate
    pip install ctypesgen
  
    ctypesgen -lhellow ${hellow}/include/*.h -o $PROJECT_DIR/__init__.py
  '';
 }