{
  pkgs ? import <nixpkgs> { },
  stdenv ? (env: if env.isLinux then pkgs.useMoldLinker env else env) pkgs.stdenv,
  ...
}:
pkgs.mkShell.override { inherit stdenv; } {
  nativeBuildInputs =
    [
      pkgs.cargo
      pkgs.clippy
      pkgs.rust-analyzer
      pkgs.rustc
      pkgs.rustfmt

      pkgs.nixfmt-rfc-style

      pkgs.pkg-config
      pkgs.openssl
    ]
    ++ pkgs.lib.optionals stdenv.isDarwin [
      pkgs.libiconv
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  # Certain Rust tools won't work without this, for example VS Code with rust analyzer
  # See https://nixos.wiki/wiki/Rust#Shell.nix_example and https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
