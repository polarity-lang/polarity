.PHONY: install
install:
	@cargo install --path app --force

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

.PHONY: update-expected
update-expected:
	@cargo test -p test-runner -- --update-expected