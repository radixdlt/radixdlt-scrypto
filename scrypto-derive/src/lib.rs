mod ast;
mod blueprint;
mod import;
mod utils;

use proc_macro::TokenStream;

/// Declares a blueprint.
///
/// The `blueprint` macro is a convenient way to define a new blueprint. It takes
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
/// #[blueprint]
/// mod counter {
///     struct Counter {
///         count: u32
///     }
///
///     impl Counter {
///         pub fn new() -> Component {
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
#[proc_macro_attribute]
pub fn blueprint(_: TokenStream, input: TokenStream) -> TokenStream {
    blueprint::handle_blueprint(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Imports a blueprint from its ABI.
///
/// This macro will generate stubs for accessing the blueprint according to
/// its ABI specification.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// import! {
/// r#"
/// {
///     "package_address": "01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876",
///     "blueprint_name": "GumballMachine",
///     "functions": [
///         {
///             "name": "new",
///             "inputs": [],
///             "output": {
///                 "type": "Custom",
///                 "name": "ComponentAddress"
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
///                     "name": "Bucket"
///                 }
///             ],
///             "output": {
///                 "type": "Custom",
///                 "name": "Bucket"
///             }
///         }
///     ]
/// }
/// "#
/// }
/// ```
#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    import::handle_import(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
