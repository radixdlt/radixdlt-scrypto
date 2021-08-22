# Scrypto

[![CI](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml/badge.svg)](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml)

Language for building DeFi apps on Radix.

## Get Started

1. Install Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. Install WebAssembly toolchain
```
rustup target add wasm32-unknown-unknown
```
3. Create a new package by copying from [the examples](./examples) and build
```
cargo build --target wasm32-unknown-unknown --release
```
4. Switch to the `simulator` folder and create a new account
```
cargo run -- new-account
```
5. To publish your package, run
```
cargo run -- publish-package /path/to/your/package
```
6. To invoke a blueprint function, run
```
cargo run -- invoke-blueprint <package_address> <blueprint> <function> <args>...
```
7. To invoke a component method, run
```
cargo run -- invoke-component <component_address> <method> <args>...
```
8. For instructions on other commands, run
```
cargo run -- help
```

## Project Layout

![](./assets/crate-dependencies.svg)

- `sbor`: Scrypto Binary Object Representation (SBOR), the data format for Scrypto.
- `sbor-derive`: SBOR derives for Rust `struct` and `enum`.
- `scrypto`: Scrypto standard library.
- `scrypto-abi`: Scrypto JSON-exportable blueprint ABI.
- `scrypto-types`: Scrypto primitive types.
- `scrypto-derive`: Derives for creating and importing Scrypto blueprints.
- `radix-engine`: Radix Engine, the Scrypto execution layer.
