{
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
  pkgs ? import nixpkgs { inherit system; },
  axe ? import (fetchTarball "https://github.com/axetechnologies/nix/archive/master.tar.gz"),
  fenix ? import (fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz") { },
}:

let
  inherit (pkgs)
    cargo-deny
    openssl
    pkg-config
    sqlite
    ;

  shell = axe.mkShell {
    inherit
      nixpkgs
      system
      pkgs
      fenix
      ;
  };

in
shell.overrideAttrs (prev: {
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    openssl
    sqlite
  ];

  nativeBuildInputs = (prev.nativeBuildInputs or [ ]) ++ [
    pkg-config
  ];

  buildInputs = (prev.buildInputs or [ ]) ++ [
    cargo-deny
    openssl
    sqlite
  ];
})
