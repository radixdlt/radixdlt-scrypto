mod ast;
mod blueprint;
mod non_fungible_data;
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

/// Derive code that describe a non-fungible data structure.
///
/// # Example
///
/// ```ignore
/// use scrypto::prelude::*;
///
/// #[derive(NonFungibleData)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     #[mutable]
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(NonFungibleData, attributes(mutable))]
pub fn non_fungible_data(input: TokenStream) -> TokenStream {
    non_fungible_data::handle_non_fungible_data(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
