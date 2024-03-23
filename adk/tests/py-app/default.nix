{ buildPythonPackage, setuptools, adk-py, adk, ... }:
buildPythonPackage rec {
  name = "py-app";
  src = ./.;
  format = "pyproject";

  nativeBuildInputs = [ setuptools ];
  propagatedBuildInputs = [ adk-py ];
}
