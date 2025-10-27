deps:
	@rustup target add wasm32-unknown-unknown
	@cargo install -f wasm-bindgen-cli --version 0.2.104
	@npm install

build: build-server build-app

run: clean-app build-server
	@npm run app --workspace=packages/app

build-server: clean-app
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
