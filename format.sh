#!/bin/bash

set -e

cd "$(dirname "$0")"

(cd scrypto; cargo fmt)
(cd scrypto-derive; cargo fmt)
(cd scrypto-tests; cargo fmt)
