{
  description = "Commit Message Editor flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        cme = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;

            meta = {
              description = "A simple editor for small commit messages on the terminal";
              longDescription = ''
                A small and simple TUI editor to create and edit commit messages fast.
              '';
              homepage = "https://github.com/Guibi1/CME";
              license = lib.licenses.mit;
              platforms = lib.platforms.all;
              mainProgram = "cme";
            };
          }
        );
      in
      {
        checks = {
          inherit cme;

          cme-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          cme-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          cme-fmt = craneLib.cargoFmt {
            inherit src;
          };

          cme-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages = {
          inherit cme;
          default = cme;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = cme;
        };

        formatter = pkgs.nixfmt;

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
        };
      }
    );
}
