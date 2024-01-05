#!/usr/bin/env bash
test -e xfunc_artifact.zip && rm xfunc_artifact.zip

DIR=$(mktemp -d)
git clone --depth 1 "$(git remote get-url origin)" "$DIR/"

pushd $DIR || exit 1

rm scripts/package_artifact.sh
rm scripts/oopsla_snippets.sh

# only keep whitelisted files in oopsla_examples/, then delete whitelist
for file in oopsla_examples/*.xfn; do
    grep -q $(basename $file) oopsla_examples/whitelist.txt || rm "$file"
done
rm oopsla_examples/whitelist.txt

# remove potential build artifacts
rm -r target/

rm -rf .git/
rm -rf .gitignore
rm -rf .github/
rm -rf .cargo/

zip -r 129.zip .

popd

mv "$DIR/129.zip" .
rm -rf "$DIR"

echo "MD5 checksum:"

md5sum 129.zip
