cargo check --all-targets --no-default-features --features std;
cargo check --manifest-path ./simulator/Cargo.toml --all-targets;
cargo check --manifest-path ./radix-engine-tests/assets/blueprints/Cargo.toml --all-targets;
cargo check --manifest-path ./assets/blueprints/Cargo.toml --all-targets;