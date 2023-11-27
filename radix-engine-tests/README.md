# Tests organization

Tests are stored in `test` subdirecotry and are separated according to execution layers using subfolders:

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

To execute tests for a specific layer run command: `cargo nextest run LAYER_NAME::` or `cargo test LAYER_NAME::`, example: `cargo nextest run kernel::`.

Running more layers at the same time is possible using command: `cargo nextest run LAYER_NAME_1:: LAYER_NAME_2::`, example `cargo nextest run kernel:: system::`.

To execute tests from specific file run command: `cargo nextest run LAYER_NAME::FILE_NAME::` or `cargo test LAYER_NAME::FILE_NAME::`, example: `cargo nextest run kernel::frame::`.
Specifying only file name could be enough, but if two layers have the same file name with tests then both will be run.

# Benches

`radix-engine-tests` project contains also some benches in `benches` subdirectory. They can be executed using command: `cargo bench`.
