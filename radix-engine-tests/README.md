# Tests organization

`test` subdirectory containing integration tests is divided according to execution layers using following subfolders:

- Application
- Blueprints
- System
- VM
- Kernel
- Db

Additional assets required by some of the tests are stored in `assets` subdirectory:

- `blueprints` all blueprints projects used during the tests
- `metering` used to store costs metering tests csv files
- `wasm` containing web assembly modules in wat file format

# Tests execution

To execute all tests run command: `cargo nextest run` or `cargo test` from `radix-engine-tests` directory or subdirectory.

To execute tests for a specific layer run command: `cargo nextest run --test LAYER_NAME` or `cargo test --test LAYER_NAME`, example: `cargo nextest run --test kernel`, `cargo nextest run --test kernel --test system`.

To execute tests from specific file run command: `cargo nextest run --test LAYER_NAME FILE_NAME`, example: `cargo nextest run --test kernel frame` (this will also run tests which include `frame` word). Alternatively name filtering can be used: `cargo test LAYER_NAME::FILE_NAME::`, example: `cargo nextest run kernel::frame::`.

# Benches

`radix-engine-tests` project contains also some benches in `benches` subdirectory. They can be executed using command: `cargo bench`.
