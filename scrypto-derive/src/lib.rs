mod blueprint;
mod import;
mod utils;

use proc_macro::TokenStream;

/// Define a new blueprint.
///
/// The `blueprint` macro is a convenient way to define a new blueprint. It should be
/// applied to two items:
/// - A `struct` which defines the structure
/// - A `impl` which defines the implementation.
///
/// This macro will derive the dispatcher method which will be responsible for handling
/// invocation according to Scrypto ABI.
///
/// # Example
/// ```ignore
/// use scrypto::constructs::*;
/// use scrypto::types::*;
/// use scrypto::*;
///
/// #[blueprint]
/// struct Counter {
///     count: u32
/// }
///
/// #[blueprint]
/// impl Counter {
///     pub fn new() -> Component {
///         Self {
///             count: 0
///         }.instantiate()
///     }
///
///     pub fn get_and_incr(&mut self) -> u32 {
///         let n = self.count;
///         self.count += 1;
///         n
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn blueprint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let output = blueprint::handle_blueprint(proc_macro2::TokenStream::from(item), true);
    TokenStream::from(output)
}

/// Import a blueprint from its ABI.
///
/// This macro will generate stubs for accessing a blueprint that complies to the
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
