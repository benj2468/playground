{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        toolchain = (fenix.packages.${system}.toolchainOf { 
          sha256 = "sha256-Mdut7K2csauBB9NL/fiEEXz+TjNHDrNc4MVhCnpm72c=";
        }).toolchain;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        adk = naersk'.buildPackage {
          src = ./adk;
          root = ./adk;
          copyLibs = true;
          nativeBuildInputs = with pkgs; [
            rust-cbindgen
            cargo-expand
          ];
          postInstall = ''
            mkdir $out/include
            cbindgen --config cbindgen.toml --crate adk --output $out/include/adk.h
          '';
        };

        adk-py = pkgs.python3Packages.callPackage ./adk/python { inherit adk; };

        c-app = pkgs.callPackage ./tests/c-app { inherit adk; };
        py-app = pkgs.python3Packages.callPackage ./tests/py-app { inherit adk-py adk; };

      in rec {
        # For `nix build` & `nix run`:
        packages = {
          default = adk;
          inherit adk-py;
          inherit c-app py-app;
        };

        # For `nix develop`:
        devShell = adk;
      }
    );
}
