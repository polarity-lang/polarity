#!/usr/bin/env bash

IMPL_DIR=`pwd`
TMP_DIR=`mktemp -d`
function cleanup {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cd $TMP_DIR
git clone --depth 1 git@github.com:ps-tuebingen/dependent-xfunctionalization.git paper-repo/
cp paper-repo/snippets/fullwidth/* $IMPL_DIR/oopsla_examples
cp paper-repo/snippets/halfwidth/* $IMPL_DIR/oopsla_examples
