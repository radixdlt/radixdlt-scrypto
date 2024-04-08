#!/bin/bash
set -e

cd "$(dirname "$0")"

# The genesis blueprints were compiled with Scrypto v1.0.0. To reproduce the same
# wasm files, the Scrypto source code must be placed at `/src` and the blueprints workspace
# source code must be placed at `/src/assets/blueprints`.
SCRYTPO_SOURCE_DIR="/tmp/radixdlt-scrypto-v1.0.0"
if [ ! -d $SCRYTPO_SOURCE_DIR ]; then
    git clone https://github.com/radixdlt/radixdlt-scrypto $SCRYTPO_SOURCE_DIR
    (cd $SCRYTPO_SOURCE_DIR; git checkout v1.0.0)
fi

IMAGE_NAME="radixdlt/scrypto-builder"
IMAGE_TAG="v1.0.0"
WORKSPACE_DIR="blueprints"
DESTINATION_DIR="."
for PACKAGE_NAME in "faucet" "genesis_helper"
do
    # Run scrypto build 
    docker run \
        --platform=linux/amd64 \
        --entrypoint=scrypto \
        -v $(realpath $SCRYTPO_SOURCE_DIR):/src \
        -v $(realpath $WORKSPACE_DIR):/src/assets/blueprints \
        $IMAGE_NAME:$IMAGE_TAG \
        build --path /src/assets/blueprints/$PACKAGE_NAME

    # Copy artifacts
    cp \
        $WORKSPACE_DIR/target/wasm32-unknown-unknown/release/$PACKAGE_NAME.{wasm,rpd} \
        $DESTINATION_DIR/
done

sha256sum *.{wasm,rpd}

# SHA256 from v1.0.0
# d35039222f6f6ea015d9fd8df6734937a64089dfcafc291071e6756b474e8775  faucet.wasm
# 87c5bef35a6e702827ef454695dbd59281b0ad76730d6aae310359b8af02e5da  genesis_helper.wasm
# 477bef3ff0d36a722e2de59670e40d85b499d57ef837fce6752523dc34809246  faucet.rpd
# b9090167a62cb8f2e15fa69e515530f050a01ec35ccf2b576e151a5ca4252994  genesis_helper.rpd