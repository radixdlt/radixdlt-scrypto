# Scrypto

[![CI](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml/badge.svg)](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml)

Language for building DeFi apps on Radix.

Documentation: https://docs-babylon.radixdlt.com/main/scrypto/introduction.html

## License

The code in this repository is released under the [Radix License, Version 1.0](LICENSE) and includes modified third party work which is reproduced here pursuant to the Apache 2.0 licensing regime.
Where third party software has been used this is identified together with the appropriate open-source licence.

Binaries, including libraries, CLIs and docker images, are licensed under the [Radix Software EULA](http://www.radixdlt.com/terms/genericEULA).

## Installation

1. Install Rust - this requires Rust 1.70+ (if rust is already installed, upgrade with `rustup update`)
    * Windows:
        * Download and install [`rustup-init.exe`](https://win.rustup.rs/x86_64)
        * Install "Desktop development with C++" with [Build Tools for Visual Studio 2019](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)
        * Install [LLVM 13.0.1](https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.1/LLVM-13.0.1-win64.exe) (make sure you tick the option that adds LLVM to the system PATH)
        * Enable git long path support:
        ```bash
        git config --system core.longpaths true
        ```   
    *  macOS:
        * Make sure you have the xcode command line tools: `xcode-select --install`.
        * Install cmake: `brew install cmake`
        * Install the Rust compiler:
        ```bash
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        ```
    * Linux:
        * Make sure a C++ compiler, LLVM and cmake is installed (`sudo apt install build-essential llvm cmake`).
        * Install the Rust compiler:
        ```bash
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        ```
2. Enable `cargo` in the current shell:
   * Windows:
       * Start a new PowerShell
   * Linux and macOS:
       ```bash
       source $HOME/.cargo/env
       ```
3. Add WebAssembly target
    ```bash
    rustup target add wasm32-unknown-unknown
    ```
4. Install simulator
    ```bash
    git clone https://github.com/radixdlt/radixdlt-scrypto.git
    cd radixdlt-scrypto
    cargo install --path ./simulator
    ```
5. (Optional) Open Scrypto documentation for later use
    ```bash
    ./doc.sh
    ```

## Getting Started

If you want a quick walkthrough of how to deploy and run some code, please see the [Run Your First Project](https://docs-babylon.radixdlt.com/main/getting-started-developers/first-component/run-first-project.html) tutorial. If you prefer to soldier through on your own, keep reading below.

### Writing Scrypto Code

1. Start by creating a new package:
```bash
scrypto new-package <package_name>
cd <package_name>
```
2. Check out the files under your current directory:
  - Source code is within `src/lib.rs`;
  - Test code is within `tests/lib.rs`.
3. Build your package:
```bash
scrypto build
```
4. Run tests:
```bash
scrypto test
```

### Playing with Radix Engine

| Action                             | Command                                                                                              |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Create an account                  | ``` resim new-account ```                                                                            |
| Change the default account         | ``` resim set-default-account <account_address> <account_public_key> ```                             |
| Create a token with fixed supply   | ``` resim new-token-fixed <amount> ```                                                               |
| Create a token with mutable supply | ``` resim new-token-mutable <minter_resource_address> ```                                            |
| Create a badge with fixed supply   | ``` resim new-badge-fixed <amount> ```                                                               |
| Create a badge with mutable supply | ``` resim new-badge-mutable <minter_resource_address> ```                                            |
| Mint resource                      | ``` resim mint <amount> <resource_address> <minter_resource_address> ```                             |
| Transfer resource                  | ``` resim transfer <amount> <resource_address> <recipient_component_address> ```                     |
| Publish a package                  | ``` resim publish <path_to_package_dir> ```                                                          |
| Call a function                    | ``` resim call-function <package_address> <blueprint_name> <function> <args> ```                     |
| Call a method                      | ``` resim call-method <component_address> <method> <args> ```                                        |
| Export the definition of a package | ``` resim export-package-definition <package_address> <output>```                                    |
| Show info about an entity          | ``` resim show <id> ```                                                                              |
| List all entities in simulator     | ``` resim show-ledger  ```                                                                           |
| Reset simulator state              | ``` resim reset ```                                                                                  |

**Note:** The commands use the default account as transaction sender.

## Compile blueprints with dockerized simulator
Follow this guide to build reproducible WASM and RDP files for your Scrypto blueprints.

### Using local docker image
The Dockerfile in the root of the repo should be work to build a docker image which will contain all the dependencies needed to be able build a blueprint using scrypto. 

Build the docker image like. From the repo root
```bash
docker build -t radixdlt/simulator .
```

Then build your package by just running
```bash
docker run -v <path-to-your-scrypto-crate>:/src radixdlt/simulator
```

### Using published docker image
If you would like to avoid building the docker image, you can skip the build step and do the second step directly, docker will automatically download the docker image we publish

Build your blueprints directly with
```bash
docker run -v <path-to-your-scrypto-crate>:/src radixdlt/simulator
```


## Project Layout

- `sbor`: The binary data format used by Scrypto.
- `sbor-derive`: Derives for encoding and decoding Rust `struct` and `enum`.
- `scrypto`: Scrypto standard library.
- `scrypto-schema`: Scrypto package schema.
- `scrypto-derive`: Derives for defining and importing Scrypto blueprints.
- `radix-engine`: The Scrypto execution engine.
- `simulator`: A simulator that run Scrypto code on a filesystem based ledger.
- `transaction`: Radix transaction manifest compiler, transaction models, signing and validationg logic.

## LFS

Assets under `assets-lfs` are stored in Git LFS.


To fetch files from LFS, install `git-lfs` first:
- MacOS
    ```
    brew install git-lfs
    ```
- Linux
    ```
    curl -s https://packagecloud.io/install/repositories/github/git-lfs/script.deb.sh | sudo bash
    ```

and then:
```
git lfs install
git lfs pull
```

## Contribute

To learn more about how to contribute to this project, read the [Contributing Guide](./CONTRIBUTING.md).
