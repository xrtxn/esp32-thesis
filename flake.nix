{
  description = "ESP32-C6 bare-metal development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # 1. Setup Oxalica Rust Toolchain for ESP32-C6
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
          # ESP32-C6 architecture
          targets = [ "riscv32imac-unknown-none-elf" ];
        };

        # 2. Setup standard RISC-V GCC from Nixpkgs
        riscvGcc = pkgs.pkgsCross.riscv32-embedded.buildPackages.gcc;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            riscvGcc
            pkgs.espflash

            # Additional tools required by mbedtls-rs build scripts
            pkgs.gnumake
            pkgs.cmake
            pkgs.python3
          ];

          # 3. Tell the Rust `cc` crate which C compiler to use for mbedtls
          CC_riscv32imac_unknown_none_elf = "${riscvGcc}/bin/riscv32-none-elf-gcc";
          CFLAGS_riscv32imac_unknown_none_elf = "-march=rv32imac -mabi=ilp32";

          shellHook = ''
            echo "🦀 ESP32-C6 Rust & C Toolchain Loaded!"
            echo "Rust version: $(rustc --version)"
            echo "GCC version: $(riscv32-none-elf-gcc --version | head -n 1)"
          '';
        };
      }
    );
}
