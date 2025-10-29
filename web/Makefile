deps:
	@rustup target add wasm32-unknown-unknown
	@cargo install -f wasm-bindgen-cli --version 0.2.104
	@npm install

build: build-server build-app

run: clean-app build-server
	@npm run app --workspace=packages/web-editor

build-server: clean-app
	@CARGO_PROFILE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=fat cargo build -p polarity-lang-lsp-wasm --target wasm32-unknown-unknown --release
	@wasm-bindgen --out-dir ./packages/lsp-web/assets/wasm --target web --typescript ../target/wasm32-unknown-unknown/release/polarity_lang_lsp_wasm.wasm

build-app:
	@npm run build --workspace=packages/web-editor

clean: clean-server clean-app

clean-server:
	@cargo clean

clean-app:
	@rm -rf packages/web-editor/dist
	@rm -rf packages/lsp-web/assets/wasm

lint:
	@npm run check --workspace=packages/web-editor
	@npm run check --workspace=packages/lsp-web
	@cargo fmt --all --check
	@cargo clippy --all -- -Dwarnings
	@npm run lint --workspace=packages/web-editor
	@npm run lint --workspace=packages/lsp-web
	@npx stylelint  "**/*.css"

format:
	@cargo fmt --all
	@npm run format --workspace=packages/web-editor
	@npm run format --workspace=packages/lsp-web
	@npx stylelint  "**/*.css" --fix
