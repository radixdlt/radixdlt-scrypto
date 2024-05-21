#!/bin/bash

set -x
set -e

(cd examples/hello-world; cargo build)
(cd examples/everything; cargo build)
(cd examples/no-std; cargo build)
(cd radix-clis; cargo build)