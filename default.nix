{
  nixpkgs ? <nixpkgs>,
  axe ? import (fetchTarball "https://github.com/axetechnologies/nix/archive/master.tar.gz"),
  ...
}@args:

let
  mailman = import ./default.nix (
    removeAttrs args [
      "crossPkgs"
      "isStatic"
      "target"
    ]
  );

in
axe.mkDefault (
  {
    src = ./.;
    version = "2.2.1";
    mkPackage = (
      {
        lib,
        pkgs,
        rustPlatform,
        defaultFeatures,
        features,
        buildPackages,
      }:

      let
        inherit (pkgs) sqlite stdenv windows;

        # NOTE: nixpkgs' mingw sqlite fails its pthread probe and compiles
        # single-threaded, defining no sqlite3_mutex_* rusqlite links against
        sqlite' =
          if stdenv.hostPlatform.isWindows then
            sqlite.overrideAttrs (old: {
              buildInputs = (old.buildInputs or [ ]) ++ [ windows.pthreads ];
            })
          else
            sqlite;

        buildFeatures = lib.splitString "," features;

        systemSqlite =
          (defaultFeatures || builtins.elem "pimdir" buildFeatures)
          && !builtins.elem "vendored" buildFeatures;

      in
      (pkgs.callPackage "${nixpkgs}/pkgs/by-name/hi/mailman/package.nix" {
        inherit lib rustPlatform buildFeatures;
        buildPackages = buildPackages // {
          inherit mailman;
        };
        installShellCompletions = false;
        installManPages = false;
        buildNoDefaultFeatures = !defaultFeatures;
      })
      # HACK: needed until the v2.1.0 derivation lands on nixpkgs's master
      .overrideAttrs
        (drv: {
          nativeBuildInputs =
            (drv.nativeBuildInputs or [ ]) ++ lib.optional systemSqlite buildPackages.pkg-config;

          buildInputs = (drv.buildInputs or [ ]) ++ lib.optional systemSqlite sqlite';

          postInstall =
            let
              inherit (pkgs) stdenv;
              exe =
                if stdenv.buildPlatform.canExecute stdenv.hostPlatform then
                  "$out/bin/mailman"
                else
                  lib.getExe mailman;
            in
            ''
              mkdir -p $out/share/{completions,man,schemas}
              ${exe} completion -d "$out"/share/completions bash elvish fish powershell zsh
              ${exe} manual -d "$out"/share/man
              ${exe} json-schema -d "$out"/share/schemas
            '';
        })
    );
  }
  // removeAttrs args [ "axe" ]
)
