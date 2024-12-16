{
  pkgs ? import <nixpkgs> { },
  stdenv ? (env: if env.isLinux then pkgs.useMoldLinker env else env) pkgs.stdenv,
  ...
}:
pkgs.mkShell.override { inherit stdenv; } {
  nativeBuildInputs = [
    pkgs.cargo
    pkgs.clippy
    pkgs.rust-analyzer
    pkgs.rustc
    pkgs.rustfmt

    pkgs.nixfmt-rfc-style

    pkgs.pkg-config
    pkgs.openssl
  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv darwin.apple_sdk.frameworks.SystemConfiguration ]);

  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

}
