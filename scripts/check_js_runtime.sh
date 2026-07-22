#!/usr/bin/env bash
set -euo pipefail

runtime_dir="runtime"
runtime_js="runtime/dist/runtime.js"

tmpdir=$(mktemp -d)
tmpdir_js="$tmpdir/runtime.js"

cleanup() {
    rm -rf "$tmpdir"
}
trap cleanup EXIT

(
    cd "$runtime_dir"
    npm run build -- --outDir "$tmpdir"
)

if ! diff -u "$runtime_js" "$tmpdir_js"; then
    echo "$runtime_js is out of date."
    echo "Run 'npm run build' in $runtime_dir and commit the result."
    exit 1
fi

echo "$runtime_js is up to date."
