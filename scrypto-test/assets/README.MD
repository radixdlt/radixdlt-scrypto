## Radix Engine - Scrypto Assets

The Test Environment is an empty scrypto component which acts as the root call-frame in the
Scrypto test runner; and we hook into it to call other things.

It could probably be removed / replaced with something else in future.

## Test environment assets

The Test Environment was compiled with the v1.1.1 deterministic scrypto builder, and deployed
as part of genesis to the ledger:

```rust
use scrypto::prelude::*;

#[blueprint]
mod test_environment {
    struct TestEnvironment {}

    impl TestEnvironment {
        pub fn run() {}
    }
}
```

The exact assets can can rebuilt with the following script, checked out against the bottlenose
version of the ledger: https://github.com/radixdlt/radixdlt-scrypto/tree/release/bottlenose

For now, to avoid confusing automated tooling, we have deleted the `Cargo.toml` files and the
`Cargo.lock` file for these genesis assets, and just kept the `lib.rs` source files around for
reference.

```bash
#!/bin/bash
set -e

cd "$(dirname "$0")"

IMAGE_NAME="radixdlt/scrypto-builder"
IMAGE_TAG="v1.1.1"
WORKSPACE_DIR="blueprints"
DESTINATION_DIR="."
for PACKAGE_NAME in "test_environment"
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
# db170d2f731cf1bb391576281e7f43629d156dbd97126d9e07e990b234f42f50  test_environment.wasm
# 481b8d309110613576be6298d3126200f470a1153dbf867193bba02a49814b66  test_environment.rpd
```