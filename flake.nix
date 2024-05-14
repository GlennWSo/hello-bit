{
  inputs = {
    nixpkgs.url = "github:NixOs/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = {
    flake-utils,
    nixpkgs,
    rust-overlay,
    crane,
    nix-filter,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (localSystem: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit overlays localSystem;};
      rust = pkgs.rust-bin.stable.latest.default.override {
        targets = ["thumbv7em-none-eabihf"];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rust;
      filter = nix-filter.lib;

      dummySrc = filter {
        root = ./.;
        include = [
          "examples/hello.rs"
          "Cargo.toml"
          "Cargo.lock"
          "memory.x"
          ".cargo"
        ];
      };

      src = filter {
        root = ./.;
        include = [
          "src"
          "examples"
          "Cargo.toml"
          "Cargo.lock"
          "memory.x"
          ".cargo"
        ];
      };
      cargoArtifacts = craneLib.buildDepsOnly {
        inherit dummySrc src;
        cargoToml = ./Cargo.toml;
        doCheck = false;
        cargoExtraArgs = "--target thumbv7em-none-eabihf";
        cargoCheckExtraArgs = "--target thumbv7em-none-eabihf";
        cargoBuildCommand = "cargo build --profile release --example hello";
      };
      crate = craneLib.buildPackage {
        inherit src cargoArtifacts;
        doCheck = false;
        cargoExtraArgs = "--target thumbv7em-none-eabihf";
        cargoCheckExtraArgs = "--target thumbv7em-none-eabihf";
        cargoBuildCommand = "cargo build --profile release";
      };
      embed = pkgs.writeShellScriptBin "embed" ''
        ${pkgs.probe-rs}/bin/probe-rs run ${crate}/bin/hello-bit --chip nRF52833_xxAA
      '';
    in {
      devShells.default = craneLib.devShell {
        name = "embeded-rs";
        inputsFrom = [crate];
        packages = with pkgs; [
          probe-rs
          rust-analyzer
          cargo-binutils
          minicom
          usbutils
          embed
        ];
      };
      apps.default = {
        type = "app";
        program = "${embed}/bin/embed";
      };
      dbg = {
        deps = cargoArtifacts;
        src = dummySrc;
        dummySrc = dummySrc;
      };
      packages = {
        default = crate;
        docs = craneLib.cargoDoc {
          inherit cargoArtifacts;
          src = dummySrc;
        };
      };
    });
}
