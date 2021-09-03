mod ast;
mod blueprint;
mod import;
mod utils;

use proc_macro::TokenStream;

/// Define the structure and implementation of a new blueprint.
///
/// The `blueprint!` macro is a convenient way to define a new blueprint. It takes
/// two arguments:
/// - A `struct` which defines the structure
/// - A `impl` which defines the implementation.
///
/// This macro will derive the dispatcher method responsible for handling invocation
/// according to Scrypto ABI.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// blueprint! {
///     struct Counter {
///         count: u32
///     }
///
///     impl Counter {
///         pub fn new() -> Address {
///             Self {
///                 count: 0
///             }.instantiate()
///         }
///
///         pub fn get_and_incr(&mut self) -> u32 {
///             let n = self.count;
///             self.count += 1;
///             n
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn blueprint(input: TokenStream) -> TokenStream {
    let output = blueprint::handle_blueprint(proc_macro2::TokenStream::from(input), true);
    TokenStream::from(output)
}

/// Import a blueprint from its ABI.
///
/// This macro will generate stubs for accessing a blueprint that complies to the
/// given ABI specification.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// import! {
/// r#"
/// {
///     "package": "05a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876",
///     "blueprint": "GumballMachine",
///     "functions": [
///         {
///             "name": "new",
///             "inputs": [],
///             "output": {
///                 "type": "Custom",
///                 "name": "scrypto::Address"
///             }
///         }
///     ],
///     "methods": [
///         {
///             "name": "get_gumball",
///             "mutability": "Mutable",
///             "inputs": [
///                 {
///                     "type": "Custom",
///                     "name": "scrypto::Tokens"
///                 }
///             ],
///             "output": {
///                 "type": "Custom",
///                 "name": "scrypto::Tokens"
///             }
///         }
///     ]
/// }
/// "#
/// }
/// ```
#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    let output = import::handle_import(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}
