#!/bin/bash

set -x
set -e

(cd examples/hello-world; cargo update)
(cd examples/everything; cargo update)
(cd examples/no-std; cargo update)
(cd radix-clis; cargo update)