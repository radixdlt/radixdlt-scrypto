#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

cd scrypto;
cargo doc --no-deps --package scrypto --package sbor;