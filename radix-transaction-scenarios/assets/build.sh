#!/bin/bash
set -e

cd "$(dirname "$0")"

if [ "$#" -ne 2 ]; then
    echo "Usage: ./build.sh <scrypto_builder_image_tag> <package_name>"
    exit 1
fi

IMAGE_NAME="radixdlt/scrypto-builder"
IMAGE_TAG="$0"
WORKSPACE_DIR="blueprints"
DESTINATION_DIR="."
for PACKAGE_NAME in $1
do
    # Run scrypto build 
    docker run \
        --platform=linux/amd64 \
        --entrypoint=scrypto \
        -v $(realpath $WORKSPACE_DIR):/src/$WORKSPACE_DIR \
        $IMAGE_NAME:$IMAGE_TAG \
        build --path /src/$WORKSPACE_DIR/$PACKAGE_NAME

    # Copy artifacts
    cp \
        $WORKSPACE_DIR/target/wasm32-unknown-unknown/release/$PACKAGE_NAME.{wasm,rpd} \
        $DESTINATION_DIR/
done