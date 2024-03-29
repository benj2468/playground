{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

        hellow = naersk'.buildPackage {
          src = ./hellow;
          copyLibs = true;
          postInstall = ''
            mkdir $out/include
            cbindgen --config cbindgen.toml --crate adk --output $out/include/adk.h
          '';
        };
        hellow-py = pkgs.callPackage ./python { inherit hellow; };

        c-app = pkgs.callPackage ./clients/c-app { inherit hellow; };
        go-app = pkgs.callPackage ./clients/go-app { inherit hellow; };
        py-app = pkgs.callPackage ./clients/py-app { inherit hellow hellow-py; };

      in rec {
        # For `nix build` & `nix run`:
        packages = {
          default = hellow;
          inherit hellow-py;
          inherit c-app py-app go-app;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo python3 ];
        };
      }
    );
}
