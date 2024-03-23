{ buildPythonPackage, gcc, setuptools, adk, ... }:
buildPythonPackage rec {
  name = "adk";
  src = ./.;
  format = "pyproject";

  nativeBuildInputs = [ gcc setuptools ];

  propagatedBuildInputs = [ adk ];

  preBuild = ''
    PROJECT_DIR=src/${name}
    mkdir -p $PROJECT_DIR

    python -m venv .venv
    source .venv/bin/activate
    pip install ctypesgen
  
    ctypesgen -L${adk}/lib -l${name} ${adk}/include/*.h -o $PROJECT_DIR/__init__.py
  '';
}
