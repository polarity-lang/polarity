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
for file in oopsla_examples/*.xfn; do
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

mkdir -p xfn-lang
cp "$DIR/build-vscode-ext/ext/vscode/xfn-0.0.1.vsix" "xfn-lang/xfn-0.0.1.vsix"
mv source-code xfn-lang
zip -r 129.zip xfn-lang

popd

mv "$DIR/129.zip" .
rm -rf "$DIR"

echo "SHA256 checksum:"

sha256sum 129.zip
