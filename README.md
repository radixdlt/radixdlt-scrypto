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
3. Add WebAssembly target
```
rustup target add wasm32-unknown-unknown
```
4. Install Radix Engine simulator
```
git clone git@github.com:radixdlt/radixdlt-scrypto.git
cd radixdlt-scrypto
cargo install --path ./simulator
```
5. (Optional) Open Scrypto reference documentation for later use
```
./doc.sh
```

**Note:** For preview release, do not delete or move the repository after installation. It will be used when resolving dependencies of Scrypto packages.

## Getting Started

### Writing Scrypto Code

1. Start by creating a new package:
```
scrypto new-package <package_name>
cd <package_name>
```
2. Check out the files under your current directory:
  - Source code is within `src/lib.rs`;
  - Test code is within `tests/lib.rs`.
3. Build your package:
```
scrypto build
```
4. Run tests:
```
scrypto test
```

### Playing with Radix Engine

| Action | Command |
|---|---|
| To create an account | ``` rev2 new-account ``` |
| To change default account | ``` rev2 set-default-account <account_address> ``` |
| To create fixed-supply resource | ``` rev2 new-resource --symbol <symbol> --supply <amount> ``` |
| To publish a package | ``` rev2 publish <package_dir_or_wasm_file> ``` |
| To call a function | ``` rev2 call-function <package_address> <blueprint_name> <function> <args> ``` |
| To call a method | ``` rev2 call-method <component_address> <method> <args> ``` |
| To export the ABI of a blueprint | ``` rev2 export-abi <package_address> <blueprint_name> ``` |
| To show info about an address | ``` rev2 show <address> ``` |

**Note:** The commands above will use the default account as transaction sender.

## Project Layout

- `sbor`: Scrypto Binary Object Representation (SBOR), the data format for Scrypto.
- `sbor-derive`: SBOR derives for Rust `struct` and `enum`.
- `scrypto`: Scrypto standard library.
- `scrypto-abi`: Scrypto JSON-exportable blueprint ABI.
- `scrypto-derive`: Derives for creating and importing Scrypto blueprints.
- `radix-engine`: Radix Engine, the Scrypto execution layer.
- `simulator`: Simulate ledger environment locally and run Scrypto code.
- `examples`: Example Scrypto code.
