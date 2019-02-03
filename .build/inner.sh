#!/bin/bash

cd /workspace

RUSTFLAGS="--remap-path-prefix=$HOME=/ --remap-path-prefix=$PWD=/" cargo build --release
strip /workspace/target/x86_64-unknown-linux-musl/release/envfmt