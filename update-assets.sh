#!/bin/bash
set -ex

# Default values
IMAGE_NAME="radixdlt/simulator"
IMAGE_TAG="latest"
REUSE_IMAGE=false

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 (--docker | --local) [--image-tag <TAG>] [--reuse-image]"
  exit 1
fi

BUILD_TYPE="$1"
shift

if [[ "$BUILD_TYPE" == "--docker" ]]; then

  # Parse command line arguments
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --reuse-image)
        REUSE_IMAGE=true
        ;;
      --image-tag)
        shift
        IMAGE_TAG="$1"
        ;;
      *)
        echo "Invalid argument: $1"
        exit 1
        ;;
    esac
    shift
  done

  IMAGE_EXISTS=$(docker images -q "$IMAGE_NAME" 2>/dev/null)

  # Check if the Docker image exists locally
  if [[ -z "$IMAGE_EXISTS" || ! "$REUSE_IMAGE" ]]; then
    echo "Docker image $IMAGE_NAME does not exist or --reuse-image flag not provided. Building..."
    docker build -t $IMAGE_NAME .
  else
    echo "Docker image $IMAGE_NAME exists, and --reuse-image flag is set. Skipping build."
  fi

  for crate_name in "faucet" "radiswap" "flash_loan" "genesis_helper" "metadata" "test_environment" "global_n_owned" "kv_store" "max_transaction"
  do
    echo "Building $crate_name..."
    docker run --entrypoint=scrypto -v $PWD:/src $IMAGE_NAME:$IMAGE_TAG build --path assets/blueprints/$crate_name
    echo "Done $crate_name!"
  done

elif [[ "$BUILD_TYPE" == "--local" ]]; then
  echo "Performing a local build..."
  # We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
  # It's also a little faster. If you wish to use the local version instead, swap out the below line.
  scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
  # scrypto="scrypto"

  cd "$(dirname "$0")/assets/blueprints"

  # See `publish_package_1mib` for how to produce the right sized wasm
  $scrypto build --disable-wasm-opt --path ../../radix-engine-tests/tests/blueprints/large_package
  cp ../../radix-engine-tests/tests/blueprints/target/wasm32-unknown-unknown/release/large_package.{wasm,rpd} ..
  ls -al ../large_package.*

  for crate_name in "faucet" "radiswap" "flash_loan" "genesis_helper" "metadata" "test_environment" "global_n_owned" "kv_store" "max_transaction"
  do
    echo "Building $crate_name..."
    (cd $crate_name; $scrypto build)

    cp \
      ./target/wasm32-unknown-unknown/release/$crate_name.{wasm,rpd} \
      ../
    echo "Done $crate_name!"
  done
else
  echo "Invalid build type: $BUILD_TYPE. Use --docker or --local."
fi
