#!/usr/bin/env bash

IMPL_DIR=$(pwd)
TMP_DIR=$(mktemp -d)
function cleanup {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cd "$TMP_DIR" || exit
git clone --depth 1 git@github.com:ps-tuebingen/dependent-xfunctionalization.git paper-repo/
rm "$IMPL_DIR/oopsla_examples"/*.xfn
mkdir "$TMP_DIR/staging"
cp paper-repo/snippets/fullwidth/* "$TMP_DIR/staging"
cp paper-repo/snippets/halfwidth/* "$TMP_DIR/staging"

while read -r example; do
  cp "$TMP_DIR/staging/$example" "$IMPL_DIR/oopsla_examples"
done < "$IMPL_DIR/oopsla_examples/whitelist.txt"

# Strip @hidden attributes from examples
find "$IMPL_DIR/oopsla_examples" -type f -name '*.xfn' -exec sed -i '/@hidden$/d' {} \;
find "$IMPL_DIR/oopsla_examples" -type f -name '*.xfn' -exec sed -i 's/@hidden //g' {} \;
