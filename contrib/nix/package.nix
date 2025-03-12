{ rustPlatform, lib, ... }:
rustPlatform.buildRustPackage rec {
  pname = "polarity";
  version = "latest";
  src = ../..;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      "tower-lsp-server-0.21.0" = "sha256-aeCc8m7zf3Kww1EBmMJFhQTYJ9lP6+R+9WzQ8yaj3Jo=";
    };
  };

  meta = {
    description = "A language with Dependendent Data and Codata Types";
    homepage = "https://polarity-lang.github.io/";
    licenses = with lib.licenses; [
      mit
      asl20
    ];
    maintainers = with lib.maintainers; [ mangoiv ];
    mainProgram = "pol";
  };
}
