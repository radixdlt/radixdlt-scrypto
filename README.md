# Scrypto

[![CI](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml/badge.svg)](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml)

Language for building DeFi apps on Radix.

Documentation: https://docs-babylon.radixdlt.com/main/scrypto/introduction.html


## Installation

1. Install Rust - this requires Rust 1.70+ (if rust is already installed, upgrade with `rustup update`)
   - Windows:
     - Download and install [`rustup-init.exe`](https://win.rustup.rs/x86_64)
     - Install "Desktop development with C++" with [Build Tools for Visual Studio 2019](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)
     - Install [LLVM 13.0.1](https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.1/LLVM-13.0.1-win64.exe) (make sure you tick the option that adds LLVM to the system PATH)
     - Enable git long path support:
     ```bash
     git config --system core.longpaths true
     ```
   - macOS:
     - Make sure you have the xcode command line tools: `xcode-select --install`.
     - Install cmake: `brew install cmake`
     - Install the Rust compiler:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
   - Linux:
     - Make sure a C++ compiler, LLVM and cmake is installed (`sudo apt install build-essential llvm cmake`).
     - Install the Rust compiler:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
2. Enable `cargo` in the current shell:
   - Windows:
     - Start a new PowerShell
   - Linux and macOS:
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

| Action                             | Command                                                                    |
| ---------------------------------- | -------------------------------------------------------------------------- |
| Create an account                  | `resim new-account`                                                        |
| Change the default account         | `resim set-default-account <account_address> <account_public_key>`         |
| Create a token with fixed supply   | `resim new-token-fixed <amount>`                                           |
| Create a token with mutable supply | `resim new-token-mutable <minter_resource_address>`                        |
| Create a badge with fixed supply   | `resim new-badge-fixed <amount>`                                           |
| Create a badge with mutable supply | `resim new-badge-mutable <minter_resource_address>`                        |
| Mint resource                      | `resim mint <amount> <resource_address> <minter_resource_address>`         |
| Transfer resource                  | `resim transfer <amount> <resource_address> <recipient_component_address>` |
| Publish a package                  | `resim publish <path_to_package_dir>`                                      |
| Call a function                    | `resim call-function <package_address> <blueprint_name> <function> <args>` |
| Call a method                      | `resim call-method <component_address> <method> <args>`                    |
| Export the definition of a package | ` resim export-package-definition <package_address> <output>`              |
| Show info about an entity          | `resim show <id>`                                                          |
| Show info about default account    | ` resim show`                                                              |
| List all entities in simulator     | `resim show-ledger `                                                       |
| Reset simulator state              | `resim reset`                                                              |

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
- Ubuntu
  ```
  curl -s https://packagecloud.io/install/repositories/github/git-lfs/script.deb.sh | sudo bash
  sudo apt-get install git-lfs
  ```

and then:

```
git lfs install
git lfs pull
```

## Contribute

To learn more about how to contribute to this project, read the [Contributing Guide](./CONTRIBUTING.md).

## License

The executable components of the Scrypto Code including libraries, CLIs and docker images, are licensed under the [Radix Software EULA](http://www.radixdlt.com/terms/genericEULA).


The code in this repository is released under the [Radix License 1.0 (modified Apache 2.0)](LICENSE):

```
Copyright 2023 Radix Publishing Ltd incorporated in Jersey, Channel Islands.

Licensed under the Radix License, Version 1.0 (the "License"); you may not use
this file except in compliance with the License.

You may obtain a copy of the License at:
https://www.radixfoundation.org/licenses/license-v1

The Licensor hereby grants permission for the Canonical version of the Work to
be published, distributed and used under or by reference to the Licensor’s
trademark Radix® and use of any unregistered trade names, logos or get-up.

The Licensor provides the Work (and each Contributor provides its Contributions)
on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
express or implied, including, without limitation, any warranties or conditions
of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A PARTICULAR
PURPOSE.

Whilst the Work is capable of being deployed, used and adopted (instantiated) to
create a distributed ledger it is your responsibility to test and validate the
code, together with all logic and performance of that code under all foreseeable
scenarios.

The Licensor does not make or purport to make and hereby excludes liability for
all and any representation, warranty or undertaking in any form whatsoever,
whether express or implied, to any entity or person, including any
representation, warranty or undertaking, as to the functionality security use,
value or other characteristics of any distributed ledger nor in respect the
functioning or value of any tokens which may be created stored or transferred
using the Work.

The Licensor does not warrant that the Work or any use of the Work complies with
any law or regulation in any territory where it may be implemented or used or
that it will be appropriate for any specific purpose.

Neither the licensor nor any current or former employees, officers, directors,
partners, trustees, representatives, agents, advisors, contractors, or
volunteers of the Licensor shall be liable for any direct or indirect, special,
incidental, consequential or other losses of any kind, in tort, contract or
otherwise (including but not limited to loss of revenue, income or profits, or
loss of use or data, or loss of reputation, or loss of any economic or other
opportunity of whatsoever nature or howsoever arising), arising out of or in
connection with (without limitation of any use, misuse, of any ledger system or
use made or its functionality or any performance or operation of any code or
protocol caused by bugs or programming or logic errors or otherwise);

A. any offer, purchase, holding, use, sale, exchange or transmission of any
cryptographic keys, tokens or assets created, exchanged, stored or arising from
any interaction with the Work;

B. any failure in a transmission or loss of any token or assets keys or other
digital artifacts due to errors in transmission;

C. bugs, hacks, logic errors or faults in the Work or any communication;

D. system software or apparatus including but not limited to losses caused by
errors in holding or transmitting tokens by any third-party;

E. breaches or failure of security including hacker attacks, loss or disclosure
of password, loss of private key, unauthorised use or misuse of such passwords
or keys;

F. any losses including loss of anticipated savings or other benefits resulting
from use of the Work or any changes to the Work (however implemented).

You are solely responsible for; testing, validating and evaluation of all
operation logic, functionality, security and appropriateness of using the Work
for any commercial or non-commercial purpose and for any reproduction or
redistribution by You of the Work. You assume all risks associated with Your use
of the Work and the exercise of permissions under this Licence.
```


The code includes modified third party work which is reproduced here pursuant to the Apache 2.0 licensing regime.
Where third party software has been used this is identified together with the appropriate open-source licence.