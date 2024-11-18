{ rustPlatform, lib, ... }:
rustPlatform.buildRustPackage rec {
  pname = "polarity";
  version = "latest";
  src = ../..;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      "codespan-0.11.1" = "sha256-Wq99v77bqSGIOK/iyv+x/EG1563XSeaTDW5K2X3kSXU=";
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
