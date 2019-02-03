#!/bin/bash

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )

docker build -t envfmt_build $DIR

docker run -t -v $DIR/../:/workspace envfmt_build

RUSTFLAGS="--remap-path-prefix=$HOME=/ --remap-path-prefix=$PWD=/" cargo build --release

mkdir -p $DIR/artifacts

cp $DIR/../target/x86_64-unknown-linux-musl/release/envfmt $DIR/artifacts/envfmt-linux-musl
cp $DIR/../target/release/envfmt $DIR/artifacts/envfmt-macos

chmod +x $DIR/artifacts/envfmt-linux-musl
chmod +x $DIR/artifacts/envfmt-macos

VERSION="v$(grep -E "^version = " $DIR/../Cargo.toml | grep -oE "\d+\.\d+\.\d+")"

hub release create $VERSION \
  -a $DIR/artifacts/envfmt-linux-musl#linux-musl \
  -a $DIR/artifacts/envfmt-macos#macOs \
  -m "$VERSION

### SHA-256 checksums
linux-musl - $(shasum -a 256 $DIR/artifacts/envfmt-linux-musl | awk '{ print $1 }')
macOs - $(shasum -a 256 $DIR/artifacts/envfmt-macos | awk '{ print $1 }')"
