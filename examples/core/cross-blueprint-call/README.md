# Cross-Blueprint Call

When the complexity of a DeFi application increases, it's sometimes impossible to put all logic into one blueprint. Instead, we need a group of modular blueprints, each including a distinct subset of the business logic.

In this example, we demonstrate two ways of calling a function or method defined in a different blueprint, depending on where it's located.

## Callee Is From A Different Package

If the function or method is from an already published package, we need to
1. Export the ABI of the blueprint using tools like `resim`
    ```
    resim export-abi <PACKAGE_ADDRESS> Airdrop
    ```
2. Import the ABI into our package, e.g.,
    ```rust
    import! {
    r#"
    {
        "package": "01bda8686d6c2fa45dce04fac71a09b54efbc8028c23aac74bc00e",
        "name": "Airdrop",
        "functions": [
            {
                "name": "instantiate_airdrop",
                "inputs": [],
                "output": {
                    "type": "Custom",
                    "name": "scrypto::core::Component",
                    "generics": []
                }
            }
        ],
        "methods": [
            {
                "name": "free_token",
                "mutability": "Immutable",
                "inputs": [],
                "output": {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                }
            }
        ]
    }
    "#
    }
    ```
Once the blueprint has been imported, we can then call any of its functions, for example,

```rust
let airdrop_component = Airdrop::instantiate_airdrop();
```

To call a method, though, we need a component address, which can be parsed from string.
```rust
let address = Address::from_str("022cf5de8153aaf56ee81c032fb06c7fde0a1dc2389040d651dfc2").unwrap();
let airdrop = Airdrop::from(address);
let received_tokens = airdrop.free_token();
```

## Callee Is From This Package

If the function or method you're calling is from this package, we can import the blueprint using Rust's `use` keyword.

In our example package, we have the following files:
```
├─ src
│   ├─ lib.rs
│   ├─ airdrop.rs
│   ├─ cross_package.rs
│   └─ intra_package.rs
├─ test
│   └─ lib.rs
└─ Cargo.toml
```

In `intra_package.rs`, we write

```rust
use crate::airdrop::Airdrop;
```

which is to import the `Airdrop` blueprint from the `airdrop` module under this crate.

Once imported, we can function/method the same way as when the callee is from a different package