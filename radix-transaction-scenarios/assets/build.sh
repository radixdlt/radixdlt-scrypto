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
# a3754ceceb76ae4493da626a0919f7bdec270e01217ee6df8386650afaa2c881  flash_loan.wasm
# d5de8e5357c22075cda4042b76d63bc607a677024cde3000800e841843701978  global_n_owned.wasm
# ab52f7b88f0bc092f8d68e34ab5ccbb3fe2ce60ae592e16838d69f94d5016156  kv_store.wasm
# 84dc4bb2649ac319748dfcac652bc1829645e6ab1b4830614911e52580eeb040  max_transaction.wasm
# 535e45f780714c281b5e235472185df5eb1287fb88cacadc117091cd53d97973  metadata.wasm
# dd64df294171e7f3fe39a92a6a9e9ebea65434f60f8ad13c278795dffcf78b0d  radiswap.wasm
# 254bfeea64aa32d55652f0674d34b20e974fb13af98b5bbd64cd73b43501a401  royalties.wasm
# 16298b21eb8287d6c6cb298c547f1e259797f4324589bc9bce235dab833b2799  flash_loan.rpd
# b1d70d96c2ffe61ae9167a550274297fc31b36038b4020eb3c07c7f844ef9c20  global_n_owned.rpd
# 0b17e886b268b8610792b98bbadc74f86d03c501ff26aeea4f5ecdcd6d1c4916  kv_store.rpd
# 82d2979c294f721da9b71b7fd147aa54b8f1a65c0b51c0ad72460cbba6366bbc  max_transaction.rpd
# 4056a12642343c7b021300a87665d6da7f0a96e0a4ff2e9c68d959eb70de688d  metadata.rpd
# c140bb5a831fc5dba1de5ec59787e085e06bf9f45719bf57f9ac51263a2d54cf  radiswap.rpd
# 8a2d7055830e0c38bfe76ab1313503d987701bca62637f07b42110d521d7a263  royalties.rpd