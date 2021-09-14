# Scrypto

[![CI](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml/badge.svg)](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml)

Language for building DeFi apps on Radix.

## Terminology

- **Package**: A collection of blueprints, built and published as a single unit.
- **Blueprint**: A template that describes the common behavior and state of its instances.
- **Component** An instance of a blueprint, which lives in the persistent state and may own resources.
- **Function**: A set of statements to perform a specific task.
- **Method**: A function attached to a component.
- **Resource**: A primitive state which can only be created once and moved.

## Installation

1. Install Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. Enable `cargo` in the current shell:
```
source $HOME/.cargo/env
```
3. Install WebAssembly toolchain
```
rustup target add wasm32-unknown-unknown
```
4. Install Radix Engine simulator
```
git clone git@github.com:radixdlt/radixdlt-scrypto.git
cd radixdlt-scrypto
cargo install --path ./simulator
```

## Getting Started

### Setting Up Workspace

1. Create a new account:
```
rev2 new-account
```
2. Create a new package:
```
rev2 new-package <package_name>
```

### Writing Scrypto Code

After setting up your workspace, check out the source file `<package>/src/lib.rs` and start writing your own blueprint.

To build and test your package, run:
```
cargo build-scrypto && cargo test-scrypto
```

### Playing with Radix Engine

- To publish your package, run
```
rev2 publish /path/to/your/package
```
- To call a function, run
```
rev2 call-function <package_address> <blueprint> <function> <args>...
```
- To call a method, run
```
rev2 call-method <component_address> <method> <args>...
```
- To show the content of an address, run
```
rev2 show <address>
```

## Project Layout

- `sbor`: Scrypto Binary Object Representation (SBOR), the data format for Scrypto.
- `sbor-derive`: SBOR derives for Rust `struct` and `enum`.
- `scrypto`: Scrypto standard library.
- `scrypto-abi`: Scrypto JSON-exportable blueprint ABI.
- `scrypto-derive`: Derives for creating and importing Scrypto blueprints.
- `radix-engine`: Radix Engine, the Scrypto execution layer.
- `simulator`: Simulate ledger environment locally and run Scrypto code.
