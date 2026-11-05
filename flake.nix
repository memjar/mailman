{
  description = "CLI to manage emails";

  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-26.05";
    };
    fenix = {
      url = "github:nix-community/fenix/monthly";
    };
    axe = {
      url = "github:axetechnologies/nix";
      flake = false;
    };
  };

  outputs =
    inputs:
    (import inputs.axe).mkFlakeOutputs inputs {
      shell = ./shell.nix;
      default = ./default.nix;
    };
}
