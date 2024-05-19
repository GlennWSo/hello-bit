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
    ...
  }:
    flake-utils.lib.eachDefaultSystem (localSystem: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit overlays localSystem;};
      rust = pkgs.rust-bin.stable.latest.default.override {
        targets = ["thumbv7em-none-eabihf"];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rust;

      fs = pkgs.lib.fileset;
      dummyFiles = fs.unions [
        ./.cargo
        (fs.fileFilter (file: file.hasExt "toml") ./.)
        (fs.fileFilter (file: file.name == "dummy.rs") ./.)
        (fs.fileFilter (file: file.name == "Cargo.lock") ./.)
      ];
      dummySrc = fs.toSource {
        root = ./.;
        fileset = dummyFiles;
      };

      srcFiles = fs.unions [
        dummyFiles
        (fs.fileFilter (file: file.hasExt "rs") ./.)
        (fs.fileFilter (file: file.name == "memory.x") ./.)
      ];
      src = fs.toSource {
        root = ./.;
        fileset = srcFiles;
      };

      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src;
        doCheck = false;
        cargoExtraArgs = "--target thumbv7em-none-eabihf";
      };
      blinky = craneLib.buildPackage rec {
        pname = "blinky";
        inherit src cargoArtifacts;
        doCheck = false;
        cargoExtraArgs = "--target thumbv7em-none-eabihf -p ${pname}";
      };
      bleBatt = craneLib.buildPackage rec {
        pname = "ble-batt";
        inherit src cargoArtifacts;
        doCheck = false;
        cargoExtraArgs = "--target thumbv7em-none-eabihf -p ${pname}";
      };
      udev_hint = ''
        "hint: make sure the microbit is connected and have mod 666 to enable flashing
        this can be achived with sudo chmod or udev settings:
          SUBSYSTEM=="usb", ATTR{idVendor}=="0d28", ATTR{idProduct}=="0204", MODE:="666""
      '';
      embedder = fw: (pkgs.writeShellScript "embed-" ''
        ${pkgs.probe-rs}/bin/probe-rs run ${fw}/bin/${fw.pname} --chip nRF52833_xxAA || echo ${udev_hint}
      '');
      embedApp = fw: {
        type = "app";
        program = "${embedder fw}";
      };
    in {
      devShells.default = craneLib.devShell {
        name = "embeded-rs";
        inputsFrom = [blinky];
        DIRENV_LOG_FORMAT = "";
        DEFMT_LOG = "info";
        shellHook = "
        ";
        packages = with pkgs; [
          probe-rs
          rust-analyzer
          cargo-binutils
          minicom
          usbutils
        ];
      };
      apps = {
        default = embedApp blinky;
        blinky = embedApp blinky;
        bleBatt = embedApp bleBatt;
      };

      dbg = {
        deps = cargoArtifacts;
        dummySrc = dummySrc.outPath;
      };
      packages = {
        inherit blinky bleBatt;
        default = blinky;
        docs = craneLib.cargoDoc {
          inherit cargoArtifacts;
          src = dummySrc;
        };
      };
    });
}
