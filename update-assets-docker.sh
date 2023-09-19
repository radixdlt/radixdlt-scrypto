#!/bin/bash
set -ex

# Default values
IMAGE_NAME="radixdlt/simulator"
IMAGE_TAG="latest"
REUSE_IMAGE=false

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



docker images -q "$IMAGE_NAME" 
IMAGE_EXISTS=$(docker images -q "$IMAGE_NAME" 2>/dev/null)

# Check if the Docker image exists locally
if [[ -z "$IMAGE_EXISTS" || ! "$REUSE_IMAGE" ]]; then
  echo "Docker image $IMAGE_NAME does not exist or --reuse-image flag not provided. Building..."
  docker build -t $IMAGE_NAME -f simulator/Dockerfile .
else
  echo "Docker image $IMAGE_NAME exists, and --reuse-image flag is set. Skipping build."
fi

for crate_name in "faucet" "radiswap" "flash_loan" "genesis_helper" "metadata" "test_environment" "global_n_owned" "kv_store"
do
  echo "Building $crate_name..."
  docker run --entrypoint=scrypto -v $PWD:/src $IMAGE_NAME:$IMAGE_TAG build --path assets/blueprints/$crate_name
  echo "Done $crate_name!"
done
