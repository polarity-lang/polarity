deps:
	@cargo install wasm-bindgen-cli --version 0.2.84
	@npm install

build: build-server build-app

build-server:
	@CARGO_PROFILE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=fat cargo build -p lsp-browser --target wasm32-unknown-unknown --release
	@wasm-bindgen --out-dir ./packages/app/assets/wasm --target web --typescript ../target/wasm32-unknown-unknown/release/lsp_browser.wasm

build-app:
	@npm run build --workspace=packages/app

clean: clean-server clean-app

clean-server:
	@cargo clean

clean-app:
	@rm -rf packages/app/dist
	@rm -rf packages/app/assets/wasm

lint:
	@cargo fmt --all --check
	@cargo clippy --all -- -Dwarnings
	@npm run lint --workspace=packages/app
	@npx stylelint  "**/*.css"

format:
	@cargo fmt --all
	@npm run format --workspace=packages/app
	@npx stylelint  "**/*.css" --fix

run:
	@npm run app --workspace=packages/app
