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
          "dummy/"
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
        cargoExtraArgs = "--target thumbv7em-none-eabihf -p dummy";
      };
      microBitFW = craneLib.buildPackage {
        inherit src cargoArtifacts;
        doCheck = false;
        cargoExtraArgs = "--target thumbv7em-none-eabihf";
      };
      udev_hint = ''
        "hint: make sure the microbit is connected and have mod 666 to enable flashing
        this can be achived with sudo chmod or udev settings:
          SUBSYSTEM=="usb", ATTR{idVendor}=="0d28", ATTR{idProduct}=="0204", MODE:="666""
      '';
      embed = pkgs.writeShellScriptBin "embed" ''
        ${pkgs.probe-rs}/bin/probe-rs run ${microBitFW}/bin/hello-bit --chip nRF52833_xxAA || echo ${udev_hint}
      '';
    in {
      devShells.default = craneLib.devShell {
        name = "embeded-rs";
        inputsFrom = [microBitFW];
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
        default = microBitFW;
        docs = craneLib.cargoDoc {
          inherit cargoArtifacts;
          src = dummySrc;
        };
      };
    });
}
