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
}
