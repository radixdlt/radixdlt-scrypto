#!/bin/bash
set -ex

# Default values
IMAGE_NAME="docker.io/radixdlt/simulator"
REUSE_IMAGE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --reuse-image)
      REUSE_IMAGE=true
      ;;
    *)
      echo "Invalid argument: $1"
      exit 1
      ;;
  esac
  shift
done

# Check if the Docker image exists locally
if ! "$REUSE_IMAGE" || ! docker image inspect "$IMAGE_NAME" &> /dev/null; then
  echo "Docker image $IMAGE_NAME does not exist or --reuse-image flag not provided. Building..."
  docker build -t $IMAGE_NAME -f simulator/Dockerfile .
else
  echo "Docker image $IMAGE_NAME exists, and --reuse-image flag is set. Skipping build."
fi

for crate_name in "faucet" "radiswap" "flash_loan" "genesis_helper" "metadata" "test_environment" "global_n_owned" "kv_store"
do
  echo "Building $crate_name..."
  docker run -it --entrypoint=scrypto -v $PWD:/src $IMAGE_NAME build --path assets/blueprints/$crate_name
  echo "Done $crate_name!"
done
