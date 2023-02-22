.PHONY: install
install:
	@cargo install --path app

.PHONY: install-editor-plugin
install-editor-plugin:
	@./scripts/install_vscode.sh

.PHONY: check
check: lint test

.PHONY: lint
lint:
	@cargo clippy --all -- -Dwarnings
	@cargo fmt --all --check
	@(cd web; make lint)

.PHONY: test
test:
	@cargo test --all
	@cargo run -p test-runner -- run
