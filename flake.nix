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
      crossSystem = "aarch64-linux";
      pkgs = import nixpkgs {inherit overlays localSystem;};
      crossPkgs = import nixpkgs {
        inherit localSystem crossSystem overlays;
      };

      rust = pkgs.rust-bin.stable.latest.default.override {
        targets = ["thumbv7em-none-eabihf"];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rust;
      # markdownFilter = path: _type: builtins.match ".*memory.x$" path != null;
      # markdownOrCargo = path: type:
      #   (markdownFilter path type) || (craneLib.filterCargoSources path type);

      # src = pkgs.lib.cleanSourceWith {
      #   src = craneLib.path ./.; # The original, unfiltered source
      #   filter = markdownOrCargo;
      # };
      filter = nix-filter.lib;
      # RUST_FLAGS = "-C link-arg=-Tlink.x -C link-arg=-Tdefmt.x";
      dummySrc = filter {
        root = ./.;
        include = [
          # "src"
          # "src/"
          "examples/hello.rs"
          "Cargo.toml"
          "Cargo.lock"
          "memory.x"
          ".cargo"
        ];
      };
      # dummySrc = craneLib.mkDummySrc {
      #   src = ./.;
      #   dummyrs = ./examples/hello.rs;
      #   extraDummyScript = "
      #     ls ${src}
      #     cp ${src}/memory.x $out/
      #   ";
      # };

      src = ./.;
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
        # cargoTestCommand = "";
        # cargoArtifacts = null;
        doCheck = false;
        cargoExtraArgs = "--target thumbv7em-none-eabihf";
        cargoCheckExtraArgs = "--target thumbv7em-none-eabihf";
        cargoBuildCommand = "cargo build --profile release";
      };
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
        ];
      };
      packages = {
        default = crate;
        consid = crate;
        deps = cargoArtifacts;
        src = dummySrc;
        dummySrc = dummySrc;
        docs = craneLib.cargoDoc {
          inherit cargoArtifacts;
          src = dummySrc;
        };
        cross.aarch64-linux.consid = let
          rust = crossPkgs.rust-bin.stable.latest.default.override {
            targets = [
              "aarch64-unknown-linux-gnu"
            ];
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain rust;
        in
          pkgs.callPackage ./default.nix
          {inherit craneLib;};
      };
    });
}
