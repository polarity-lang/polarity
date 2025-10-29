.PHONY: install
install:
	@cargo build --release
	@cargo install --offline --locked --path app --force

.PHONY: check
check: lint test

.PHONY: lint
lint:
	@cargo clippy --all -- -Dwarnings
	@cargo fmt --all --check
	@(cd web; make lint)

.PHONY: test
test:
	@cargo test --workspace

.PHONY: bench
bench:
	@cargo run -p polarity-bench -- --bench

.PHONY: update-expected
update-expected:
	@cargo test -p test-runner -- --update-expected


.PHONY: coverage
coverage:
	@echo "Make sure to install via cargo install cargo-llvm-cov first"
	@cargo llvm-cov --workspace --html
	@cargo llvm-cov --workspace --open

.PHONY: package-quick
package-quick:
	@cargo package --workspace --no-verify --exclude polarity-bench --exclude test-runner --exclude polarity-lang-lsp-wasm

.PHONY: package
package:
	@cargo package --workspace --no-verify --exclude polarity-bench --exclude test-runner --exclude polarity-lang-lsp-wasm
