#!/bin/bash

cd /workspace

RUSTFLAGS="--remap-path-prefix=$HOME=/ --remap-path-prefix=$PWD=/" cargo build --release