{
  pkgs ? import <nixpkgs> { },
  ...
}:
{
  polarity = pkgs.callPackage ./package.nix { };
  polarity-static = pkgs.pkgsStatic.callPackage ./package.nix { };
}
