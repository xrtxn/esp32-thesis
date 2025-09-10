{ lib, stdenv, esp-idf, jq, writeShellApplication }:

{ name, target, src }: let

  flash = writeShellApplication {
    name = "${name}-flash";
    text = ''
      set -e
      
      SCRIPT_DIR="$(dirname "$(readlink -f "''${BASH_SOURCE[0]}")")"
      cd "$SCRIPT_DIR/../build"

      eval "$(
        jq -r '.extra_esptool_args | to_entries | map("\(.key)=\(.value|@sh)") | .[]' "flasher_args.json"
      )"

      stubarg=""
      if [ "$stub" = "false" ]; then
        stubarg="--no-stub"
      fi

      ${esp-idf}/python-env/bin/python3 -m esptool "$@" --chip "$chip" --before "$before" --after "$after" $stubarg write_flash "@flash_args"
    '';

    checkPhase = "";

    runtimeInputs = [
      esp-idf
      jq
    ];
  };

in

stdenv.mkDerivation {
  inherit name;

  buildInputs = [
    esp-idf
  ];

  phases = [ "buildPhase" ];

  buildPhase = ''
    cp -r ${src}/* .
    chmod -R +w .

    # The build system wants to create a cache directory somewhere in the home
    # directory, so we make up a home for it.
    mkdir temp-home
    export HOME=$(readlink -f temp-home)

    # idf-component-manager wants to access the network, so we disable it.
    export IDF_COMPONENT_MANAGER=0

    idf.py --preview set-target ${target}
    export NINJAFLAGS=-v
    idf.py build

    mkdir $out
    cp -r * $out

    mkdir $out/bin
    cp ${lib.getExe flash} $out/bin/
    '';

  meta = {
    mainProgram = "${name}-flash";
  };
}

