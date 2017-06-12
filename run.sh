#!/usr/bin/env sh
./rustup.sh --default-toolchain nightly-2017-06-08
cargo build --release
./target/release/rusty_ems
