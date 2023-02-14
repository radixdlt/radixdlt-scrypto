# Hello World!

A good "Hello, World!" example provides the simplest possible piece of code to understand the basics of a new language. However, Scrypto isn't just a typical language – it is specialized for the management of assets on a decentralized network. So rather than just printing "Hello, World!" to a console, our example will hand out a token! Hopefully you'll get a taste of how asset-oriented programming with Scrypto for DeFi works.

## File Structure

For every new Scrypto package, there are mainly three files/folders:
- The `src` folder, which contains all the source code;
- The `test` folder, which contains all the test code;
- The `Cargo.toml` file which specifies all the dependencies and compile configurations.

## Blueprint

A *blueprint* is the code that defines a shared data structure and implementation. Multiple blueprints are grouped into a *package*.

In this example, we have only one blueprint in the package called `Hello`, which defines:

* The state structure of all `Hello` components (a single *vault*, which is a container for *resources*);
* A function `instantiate_hello`,  which instantiates a `Hello` component;
* A method `free_token`, which returns a bucket of `HelloToken` each time invoked.

```rust
use scrypto::prelude::*;

#[blueprint]
mod hello {
    struct Hello {
        sample_vault: Vault,
    }

    impl Hello {
         pub fn instantiate_hello() -> Component {
            // stripped
         }

         pub fn free_token(&mut self) -> Bucket {
            // stripped
         }
    }
}
```

## Component

The way to instantiate a component is through the `instantiate()` method on the state structure, after providing the initial values for all the fields.

```rust
Self {
    sample_vault: Vault::with_bucket(my_bucket),
}
.instantiate()
```

## Resource, Vault and Bucket

In Scrypto, assets like tokens, NFTs, and more are not implemented as blueprints or components. Instead, they are types of *resources* that are configured and requested directly from the system.

To define a new resource, we use the `ResourceBuilder`, specifying the metadata and initial supply. We can use the `ResourceBuilder` to create a simple fungible-supply token called `HelloToken` like this:

```rust
let my_bucket: Bucket = ResourceBuilder::new_fungible()
    .metadata("name", "HelloToken")
    .metadata("symbol", "HT")
    .mint_initial_supply(1000);
```

Once created, the 1000 resource-based `HelloToken` tokens are held in transient container `my_bucket`. To permanently store the created resources, we need to put them into a `Vault` like this:
```rust
let vault: Vault = Vault::with_bucket(my_bucket);
```

## How to Play?

1. Create a new account, and save the account component address
```
resim new-account
```
2. Publish the package, and save the package ID
```
resim publish .
```
3. Call the `instantiate_hello` function to instantiate a component, and save the component address
```
resim call-function <PACKAGE_ADDRESS> Hello instantiate_hello
```
4. Call the `free_token` method of the component we just instantiated
```
resim call-method <COMPONENT_ADDRESS> free_token
```
5. Check out our balance
```
resim show <ACCOUNT_COMPONENT_ADDRESS>
```