{ buildPythonPackage, setuptools, adk-py, ... }:
buildPythonPackage rec {
  name = "py-app";
  src = ./.;
  format = "pyproject";

  nativeBuildInputs = [ setuptools ];
  propagatedBuildInputs = [ adk-py ];
}
