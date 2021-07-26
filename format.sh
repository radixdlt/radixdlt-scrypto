#!/bin/bash

set -e

(cd scrypto; cargo fmt)
(cd scrypto-derive; cargo fmt)
(cd scrypto-tests; cargo fmt)