#!/bin/bash

set -x
set -e

(cd assets/blueprints; cargo update)
(cd simulator; cargo update)
(cd examples/hello-world; cargo update)
(cd examples/everything; cargo update)
(cd examples/no-std; cargo update)