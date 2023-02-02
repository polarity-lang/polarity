#!/usr/bin/env bash

set -e

if ! [ -x "$(command -v code)" ]; then
  echo 'Skipping installation of VSCode plugin: command "code" not found.' >&2
  exit 0
fi

if ! [ -x "$(command -v npm)" ]; then
  echo 'Skipping installation of VSCode plugin: command "npm" not found.' >&2
  exit 0
fi

(cd ext/vscode; npm install; npx vsce package)
code --install-extension ext/vscode/xfn-0.0.1.vsix
