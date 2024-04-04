#!/bin/bash
set -e

cd "$(dirname "$0")"

IMAGE_NAME="radixdlt/scrypto-builder"
IMAGE_TAG="v1.1.1"
WORKSPACE_DIR="blueprints"
DESTINATION_DIR="."
for PACKAGE_NAME in "flash_loan" "global_n_owned" "kv_store" "max_transaction" "metadata" "radiswap" "royalties"
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

sha256sum *.{wasm,rpd}

# SHA256
# 1ae5123ca06271963d7b884a9a80ba3d8cae2ca682cbe900da07f47e1b55b448  flash_loan.wasm
# 1c91883a19638d1341e0bac08eb1d58ca031bed6de24b3c75dea74ea938582ec  global_n_owned.wasm
# 9a5a69772cb51db410d929c9412c621e60683ed6e9320bdb7fa85917d017926c  kv_store.wasm
# a173a1012bd14a564d83807a7ce49ec64da8c06f993c4c50548f89dd6e9310e8  max_transaction.wasm
# 6bd41c5b0ddd507844fad7391491102f5d2db6dccac780db6c4b34e045e9de08  metadata.wasm
# 4deb638a95f6bbb200fb97beb01e9114270717a3d70b3309d9469f6f93c67ad7  radiswap.wasm
# e28c607a217b798ec8e4d4c06420d6061ac5c8c9d4b2b9e280ab98f2d4cc733c  royalties.wasm
# 16298b21eb8287d6c6cb298c547f1e259797f4324589bc9bce235dab833b2799  flash_loan.rpd
# b1d70d96c2ffe61ae9167a550274297fc31b36038b4020eb3c07c7f844ef9c20  global_n_owned.rpd
# 0b17e886b268b8610792b98bbadc74f86d03c501ff26aeea4f5ecdcd6d1c4916  kv_store.rpd
# 82d2979c294f721da9b71b7fd147aa54b8f1a65c0b51c0ad72460cbba6366bbc  max_transaction.rpd
# 4056a12642343c7b021300a87665d6da7f0a96e0a4ff2e9c68d959eb70de688d  metadata.rpd
# c140bb5a831fc5dba1de5ec59787e085e06bf9f45719bf57f9ac51263a2d54cf  radiswap.rpd
# 8a2d7055830e0c38bfe76ab1313503d987701bca62637f07b42110d521d7a263  royalties.rpd