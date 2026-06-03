{
  description = "ESP32 thesis project using esp-rs-nix for Rust development and mbedtls support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    esp-rs-nix = {
      url = "github:xrtxn/esp-rs-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      esp-rs-nix,
      ...
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          esp-rs = esp-rs-nix.packages.${system}.esp-rs;

          # Add the standard RISC-V GCC cross-compiler specifically for compiling Mbed TLS C source
          riscvGcc = pkgs.pkgsCross.riscv32-embedded.buildPackages.gcc;
        in
        {
          default = pkgs.mkShell {
            name = "esp32-thesis";

            buildInputs = [
              esp-rs
              pkgs.rustup
              pkgs.espflash
              pkgs.rust-analyzer
              pkgs.pkg-config
              pkgs.stdenv.cc

              pkgs.cargo-bloat
              pkgs.SDL2

              pkgs.bacon
              pkgs.pre-commit
              pkgs.esp-generate
              pkgs.probe-rs-tools

              # Required tools for the mbedtls-rs build script (cc crate)
              riscvGcc
              pkgs.gnumake
              pkgs.cmake
              pkgs.python3
            ];

            # Tell the Rust `cc` crate which C compiler to use for mbedtls on ESP32-C6
            CC_riscv32imac_unknown_none_elf = "${riscvGcc}/bin/riscv32-none-elf-gcc";
            CFLAGS_riscv32imac_unknown_none_elf = "-march=rv32imac -mabi=ilp32";

            shellHook = ''
              # Add a prefix to the shell prompt
              export PS1="(esp-rs)$PS1"

              export AP_SSID="Thesis-MM"
              export AP_PASS="Thesis2026"

              # This variable is important - it tells rustup where to find the esp toolchain,
              # without needing to copy it into your local ~/.rustup/ folder.
              export RUSTUP_TOOLCHAIN=${esp-rs}

              # Set RUST_SRC_PATH for build-std to find the library sources
              export RUST_SRC_PATH=${esp-rs}/lib/rustlib/src/rust/library

              # Override where Cargo looks for stdlib sources (needed for build-std)
              export __CARGO_TESTS_ONLY_SRC_ROOT=${esp-rs}/lib/rustlib/src/rust/library

              # Fetch Pico CSS if not already present
              if [ -f "$PWD/flake.nix" ] && [ ! -f "$PWD/web/static/pico.min.css" ]; then
                mkdir -p "$PWD/web/static"
                ${pkgs.curl}/bin/curl -fsSL \
                  https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css \
                  -o "$PWD/web/static/pico.min.css"
              fi

              echo "🦀 ESP32 Thesis Environment Loaded!"
              echo "RISC-V GCC for mbedtls ready: $(riscv32-none-elf-gcc --version | head -n 1)"
            '';
          };
        }
      );
    };
}
