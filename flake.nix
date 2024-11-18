{
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        { pkgs, config, ... }:
        {
          devShells.default = import ./contrib/nix/shell.nix { inherit pkgs; };
          packages = import ./contrib/nix/default.nix { inherit pkgs; } // {
            default = config.packages.polarity;
          };
          formatter = pkgs.nixfmt-rfc-style;
        };
    };
}
