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
            cp -r include $out
          '';
        };

        hellow-c = pkgs.callPackage ./clients/c-app { inherit hellow; };
        hellow-py = pkgs.callPackage ./clients/python { inherit hellow; };

      in rec {
        # For `nix build` & `nix run`:
        packages = {
          default = hellow;
          inherit hellow-c hellow-py;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo python3 ];
        };
      }
    );
}
