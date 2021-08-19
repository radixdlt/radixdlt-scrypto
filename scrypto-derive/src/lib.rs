mod ast;
mod component;
mod import;
mod utils;

use proc_macro::TokenStream;

/// Define the structure and implementation of a new component.
///
/// The `component!` macro is a convenient way to define a new component. It takes
/// two arguments:
/// - A `struct` which defines the structure of the component state
/// - A `impl` which defines the methods of the component.
///
/// This macro will derive the dispatcher method responsible for handling
/// cross-component invocation and data exchange.
///
/// # Example
/// ```ignore
/// use scrypto::constructs::*;
/// use scrypto::types::*;
/// use scrypto::*;
///
/// component! {
///     struct Counter {
///         count: u32
///     }
///
///     impl Counter {
///         pub fn new() -> Address {
///             Component::new("Counter", Self {
///                 count: 0
///             }).into()
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
pub fn component(input: TokenStream) -> TokenStream {
    let output = component::handle_component(proc_macro2::TokenStream::from(input), true);
    TokenStream::from(output)
}

/// Import a component from ABI.
///
/// This macro will generate stubs for accessing a component that complies to the
/// given ABI specification.
///
/// # Example
/// ```ignore
/// use scrypto::*;
///
/// import! { "/path/to/abi.json" };
/// ```
#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    let output = import::handle_import(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}
