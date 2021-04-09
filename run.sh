#!/usr/bin/env sh

cargo run --package fsmetrics --bin fsmetrics
sudo ./target/debug/fsmetrics
