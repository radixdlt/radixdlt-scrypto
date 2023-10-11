#!/bin/bash

# List all fuzz targets
cargo metadata --format-version=1 | jq -r '.packages[] | select(.source == null) .targets[] | select(.kind[] | contains("bin")).name'
