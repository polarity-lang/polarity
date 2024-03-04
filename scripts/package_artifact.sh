#!/usr/bin/env bash

set -o pipefail -o errexit -o nounset

test -e xfunc_artifact.zip && rm xfunc_artifact.zip

SOURCE_GIT_URL="https://github.com/polarity-lang/oopsla24"
VSCODE_GIT_URL="https://github.com/polarity-lang/vscode"
DIR=$(mktemp -d)
git clone --depth 1 "$SOURCE_GIT_URL" "$DIR/source-code"

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

git clone --depth 1 "$VSCODE_GIT_URL" "$DIR/build-vscode-ext"

pushd "$DIR/build-vscode-ext" || exit 1
npm install
vsce package --allow-missing-repository

popd

mkdir -p polarity-lang
cp "$DIR/build-vscode-ext/polarity-0.0.1.vsix" "polarity-lang/polarity-0.0.1.vsix"
mv source-code polarity-lang
zip -r polarity-lang.zip polarity-lang

popd

mv "$DIR/polarity-lang.zip" .
rm -rf "$DIR"

echo "SHA256 checksum:"

sha256sum polarity-lang.zip
