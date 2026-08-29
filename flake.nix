{
  description = "Recall your answers to Claude Code's questions from session transcripts through a typed Datomic query";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromToolchainFile pkgs {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
        };

        inherit (rust) craneLib toolchain;
        # Keep transcript fixtures and the authored Ethos map in the build
        # sandbox; crane's default filter strips these non-Rust inputs.
        src = rust.cleanSource {
          root = ./.;
          extraFilters = [
            (path: _type: pkgs.lib.hasSuffix ".jsonl" path)
            (path: _type: pkgs.lib.hasSuffix ".ethos" path)
          ];
        };
        commonArgs = {
          inherit src;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        packages.default = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        checks = {
          build = craneLib.cargoBuild (commonArgs // {
            inherit cargoArtifacts;
          });
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });
          doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          ethos-source = pkgs.runCommand "claude-answers-ethos-source" { } ''
            test -f ${src}/claude-answers.ethos
            touch "$out"
          '';
        };

        devShells.default = pkgs.mkShell {
          name = "claude-answers";
          packages = [
            pkgs.jujutsu
            pkgs.pkg-config
            toolchain
          ];
        };
      }
    );
}
