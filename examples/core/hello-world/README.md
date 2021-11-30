# Hello World!

Welcome to this "Hello, World!" example in Scrypto! Different from many other programming languages where you get a "Hello, World!" printed to your console, we will do more meaningful stuff in this example. Hopefully, you'll get a taste of how asset-oriented programming fits the DeFi world.

## File Structure

For every new Scrypto package, there are mainly three files/folders:
- The `src` folder, which contains all the source code;
- The `test` folder, which contains all the test code;
- The `Cargo.toml` file which specifies all the dependencies and compile configurations.

Let's jump straight into the source code.

## Blueprint

Blueprint is the code that defines a shared data structure and implementation, and multiple blueprints are grouped into a package.


In this example, we have only one blueprint called `Hello`. Every instance of `Hello` blueprint (we call it "component") will have a data structure which contains a single vault, and an implementation which has a single method `free_token`. 

```rust
use scrypto::prelude::*;

blueprint! {
    struct Hello {
        sample_vault: Vault,
    }

    impl Hello {
         pub fn new() -> Component {
            // stripped
         }

         pub fn free_token(&mut self) -> Bucket {
            // stripped
         }
    }
}
```

In the implementation, we also have a `new` function which does not accept any parameter and returns a component. The only difference between function and method is that a function does not require a `self` reference to the structure.

## Component

The way to instantiate a component is through the `instantiate()` method after providing the initial value for all the fields in the structure.

```rust
Self {
    sample_vault: Vault::with_bucket(my_bucket),
}
.instantiate()
```

## ResourceDef, Vault and Bucket

In Scrypto, resources are the abstraction of physical assets, like tokens, badges and NFTs. 

To define a new resource, we use the `ResourceBuilder` by specifying the metadata and initial supply:
```rust
let my_bucket: Bucket = ResourceBuilder::new_fungible(0)
    .metadata("name", "HelloToken")
    .metadata("symbol", "HT")
    .initial_supply_fungible(1000);
```

Once created, the resource is held in transient container `my_bucket`. To permanently store the created resource, we need to put it into a `Vault` like in the example:
```rust
let vault: Vault = Vault::with_bucket(my_bucket);
```

## How to Play?

1. Create a new account, and save the account address
```
resim new-account
```
2. Publish the package, and save the package address
```
resim publish .
```
3. Call the `new` function to instantiate a component, and save the component address
```
resim call-function <PACKAGE_ADDRESS> Hello new
```
4. Call the `free_token` method of the component we just instantiated
```
resim call-method <COMPONENT_ADDRESS> free_token
```
5. Check out our balance
```
resim show <ACCOUNT_ADDRESS>
```