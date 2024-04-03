#!/bin/bash
set -e

# Default values
IMAGE_NAME="radixdlt/scrypto-builder"
IMAGE_TAG="latest"
REUSE_IMAGE="false"
BUILD_TYPE="docker"
WORKSPACE_DIR=""
PACKAGE_NAME=""
DESTINATION_DIR=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --reuse-image)
      REUSE_IMAGE="true"
      ;;
    --image-tag)
      shift
      IMAGE_TAG="$1"
      ;;
    --docker)
      BUILD_TYPE="docker"
      ;;
    --local)
      BUILD_TYPE="local"
      ;;
    --workspace)
      shift
      WORKSPACE_DIR="$1"
      ;;
    --package)
      shift
      PACKAGE_NAME="$1"
      ;;
    --destination)
      shift
      DESTINATION_DIR="$1"
      ;;
    *)
      echo "Invalid argument: $1"
      exit 1
      ;;
  esac
  shift
done

if [[ "$WORKSPACE_DIR" == "" ]]; then
  echo "Please provide workspace dir with --workspace"
  exit 1
fi

if [[ "$PACKAGE_NAME" == "" ]]; then
  echo "Please provide package name with --package"
  exit 1
fi

if [[ "$DESTINATION_DIR" == "" ]]; then
  echo "Please provide destination dir with --destination"
  exit 1
fi

if [[ "$BUILD_TYPE" == "docker" ]]; then

  # Check if the Docker image exists locally
  if [[  "$REUSE_IMAGE" == "false" ]]; then
    echo "--reuse-image flag not provided. Building..."
    docker build -t $IMAGE_NAME $(dirname "$0")
  else
    echo "--reuse-image flag is set. Skipping build."
  fi

  echo "Building $WORKSPACE_DIR/$PACKAGE_NAME..."

  # The genesis blueprints were compiled with Scrypto v1.0.0. To reproduce the same
  # wasm file, the Scrypto source code must be placed at `/src` and the blueprints workspace
  # source code must be placed at `/src/assets/blueprints`.
  SCRYTPO_SOURCE_DIR="/tmp/radixdlt-scrypto-v1.0.0"
  if [ ! -d $SCRYTPO_SOURCE_DIR ]; then
    git clone https://github.com/radixdlt/radixdlt-scrypto $SCRYTPO_SOURCE_DIR
    (cd $SCRYTPO_SOURCE_DIR; git checkout v1.0.0)
  fi
  
  # Run scrypto build 
  docker run \
    --platform=linux/amd64 \
    --entrypoint=scrypto \
    -v $(realpath $SCRYTPO_SOURCE_DIR):/src \
    -v $(realpath $WORKSPACE_DIR):/src/assets/blueprints \
    $IMAGE_NAME:$IMAGE_TAG \
    build --path /src/assets/blueprints/$PACKAGE_NAME

  # Copy artifacts to destination directory
  cp \
    $WORKSPACE_DIR/target/wasm32-unknown-unknown/release/$PACKAGE_NAME.{wasm,rpd} \
    $DESTINATION_DIR/

  echo "Done!"

elif [[ "$BUILD_TYPE" == "local" ]]; then
  echo "Performing a local build..."
  # We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
  # It's also a little faster. If you wish to use the local version instead, swap out the below line.
  scrypto="scrypto"

  echo "Building $WORKSPACE_DIR/$PACKAGE_NAME..."
  (cd $WORKSPACE_DIR/$PACKAGE_NAME; $scrypto build)

  cp \
    $WORKSPACE_DIR/target/wasm32-unknown-unknown/release/$PACKAGE_NAME.{wasm,rpd} \
    $DESTINATION_DIR/
  echo "Done"
else
  echo "Invalid build type: $BUILD_TYPE. Use --docker or --local."
fi
