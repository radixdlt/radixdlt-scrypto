#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

cd radix-engine;
cargo doc --no-deps --package scrypto --package sbor --package radix-engine;