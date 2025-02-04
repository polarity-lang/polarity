{ rustPlatform, lib, ... }:
rustPlatform.buildRustPackage rec {
  pname = "polarity";
  version = "latest";
  src = ../..;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      "tower-lsp-0.20.0" = "sha256-f3S2CyFFX6yylaxMoXhB1/bfizVsLfNldLM+dXl5Y8k=";
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
