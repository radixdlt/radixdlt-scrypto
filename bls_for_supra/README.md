# bls_for_supra
  Simple CLI tool to demonstrate how to work with Scrypto blueprints.
  The works with Enkinet test network. It communicates with it via Gateway HTTP REST API.

  The tool by default uses CryptoScrypto package published at address:
    package_tdx_21_1p5hg2nmhxthzz8hdhqaclx376pq77yv8zfagq6h9hxk6tw5sdmx090


  Enkinet network:
  - gateway URL:
    https://enkinet-gateway.radixdlt.com/
  - dashboard URL:
    https://enkinet-dashboard.rdx-works-main.extratools.works/

For more details try:
```
cargo run -- --help
```

## Gateway status
command:
```
cargo run -- gateway-status
```

## Keccak hash
- get hash of default message by calling default CryptoScrypto package
```
cargo run -- keccak-hash
```

- get hash of given message by calling given CryptoScrypto package
```
cargo run -- keccak-hash -a package_tdx_21_1pkt7zdllsneytdc9g60xn9jjhhwx7jaqxmeh58l4dwyx7rt5z9428f -m "abc"
```
- help
```
cargo run -- keccak-hash --help
```

## BLS verify
- run with default parameters
```
cargo run -- bls-verify
```
- help
```
cargo run -- bls-verify --help
```

## Other BLS related methods
```
cargo run -- bls-aggregate-verify
cargo run -- bls-fast-aggregate-verify
cargo run -- bls-signature-aggregate
```

## Publish package
- run with default parameters
```
cargo run -- publish-package
```
- help
```
cargo run -- publish-package --help
```

# CryptoScrypto blueprint
  located in: `bls_for_supra/crypto_scrypto`

  'CryptoScrypto' Scrypto blueprint that demonstrates how to use
  CryptoUtils from Radix Engine to:
  - get Keccak256 hash
  - perform BLS signature verification

  In order to build a package use command
  ```
  scrypto build
  ```
  The command should produce `crypto_scrypto.[wasm,rpd]` files in the target/ folder

## CryptoScrypto binary files
  path: `bls_for_supra/crypto_scrypto/crypto_scrypto.[wasm,rpd]`

  These are exemplary outputs of scrypto build' in crypto_scrypto blueprint

  These files might be used to publish a CryptoScrypto package in the test network
  without building the above blueprint

