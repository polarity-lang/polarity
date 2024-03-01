#!/usr/bin/env bash
test -e xfunc_artifact.zip && rm xfunc_artifact.zip

GIT_URL="$(git remote get-url origin)"
DIR=$(mktemp -d)
git clone --depth 1 "$GIT_URL" "$DIR/source-code"

pushd "$DIR" || exit 1
pushd "$DIR/source-code" || exit 1

rm scripts/package_artifact.sh
rm scripts/oopsla_snippets.sh

# only keep whitelisted files in oopsla_examples/, then delete whitelist
for file in oopsla_examples/*.pol; do
    grep -q $(basename $file) oopsla_examples/whitelist.txt || rm "$file"
done
rm oopsla_examples/whitelist.txt

rm -rf .git/
rm -rf .gitignore
rm -rf .github/
rm -rf .cargo/

popd

git clone --depth 1 "$GIT_URL" "$DIR/build-vscode-ext"

pushd "$DIR/build-vscode-ext" || exit 1
cd ext/vscode
npm install
vsce package --allow-missing-repository

popd

mkdir -p pol-lang
cp "$DIR/build-vscode-ext/ext/vscode/pol-0.0.1.vsix" "pol-lang/pol-0.0.1.vsix"
mv source-code pol-lang
zip -r 129.zip pol-lang

popd

mv "$DIR/129.zip" .
rm -rf "$DIR"

echo "SHA256 checksum:"

sha256sum 129.zip
