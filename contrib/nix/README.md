# How to Use the Nix Code

## Obtaining Nix

Refer to one of the sets of instructions to install nix:
- [lix.systems/install](https://lix.systems/install/)
- [nixos.org/install](https://nixos.org/download/)

Make sure you enable flakes support, if you do *not* know what you are doing.

Optionally, install direnv and the nix-direnv integration
- [github.com/direnv/direnv](https://github.com/direnv/direnv)
- [github.com/nix-community/nix-direnv](https://github.com/nix-community/nix-direnv)

## Using Nix to Develop Polarity

We provide a development environment that should contain everything to build polarity and work on it. To enter a development shell containing all the tools you need, run:
```sh
nix develop
```

If you do not want to wait for evaluation of the nix code every time you enter the directory, using direnv and nix-direnv you can add a `.envrc` file in the root directory of your checkout, containing
```sh
use flake . -Lv
```
After that, you only have to run `direnv allow` which will then allow direnv to enter the development shell automatically every time you enter the polarity directory.

## Building Polarity Using Nix

To build polarity with nix
- in your local checkout, run:
  ```sh
  nix build
  ```
- if you do not have a local checkout, run:
  ```sh
   nix build github:polarity-lang/polarity
  ```
- if you do not care for the latest version, nixpkgs also has polarity:
  ```sh
  nix build nixpkgs#polarity
  ```

This will put the `pol` executable in the `result/bin` symlink relative to your current working directory.

If you want to directly run polarity, replace `build` with `run` and if you want to temporarily add polarity to your `$PATH`, replace `build` with `shell`.

If you want to install polarity globally using nix, use `nix profile install` instead of `nix build`.

## Contributing to the Nix Code

Before opening a merge request with changes to the nix code, please check the following:

- all `*.nix` files except the `flake.*` files live in the `contrib/nix` directory
- all of the nix code keeps working with both legacy nix and flakes
- your changes have been formatted with `nix fmt`, or if you do not use flakes, `nix-fmt` (rfc-style)
- your changes use existing nixpkgs infrastructure where possible
- your changes do not break any of the existing workflows and everything works as normal for non-nix users
- your changes do not require changes in regular non-nix development workflows
- your changes do not introduce any unnecessary IFD (see [the nix manual for an explanation of what IFD is](https://nix.dev/manual/nix/latest/language/import-from-derivation))
