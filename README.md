# Scrypto

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
3. Create a new project from [the examples](./examples) and build
```
cargo build --target wasm32-unknown-unknown --release
```
4. Navigate to Radix Engine and publish our blueprint
```
cargo run -- publish /path/to/<project_name>/target/wasm32-unknown-unknown/release/<project_name>.wasm
```
5. Invoke a method of the blueprint
```
cargo run -- call <blueprint_address> <component_name> <method_name> <arg>...
```

## Project Layout

![](./assets/crate-dependencies.svg)

- `sbor`: Scrypto Binary Object Representation (SBOR), format for all data exchanges.
- `sbor-derive`: SBOR derives for Rust `struct` and `enum`.
- `scrypto`: Scrypto standard library.
- `scrypto-abi`: Scrypto JSON-compatible component ABI.
- `scrypto-types`: Scrypto primitive types.
- `scrypto-derive`: Scrypto derives for creating and importing components.
- `radix-engine`: Radix Engine, the Scrypto execution layer.
