mod describe;
mod non_fungible_data;
mod scrypto;
mod utils;

use proc_macro::TokenStream;

/// Derive code that describes this data structure.
///
/// Note that this derive doesn't work with recursive type, such as
/// ```ignore
/// struct A {
///     array: Vec<A>
/// }
/// ```
#[proc_macro_derive(Describe, attributes(skip))]
pub fn describe(input: TokenStream) -> TokenStream {
    describe::handle_describe(proc_macro2::TokenStream::from(input))
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
///     #[scrypto(mutable)]
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(NonFungibleData, attributes(scrypto))]
pub fn non_fungible_data(input: TokenStream) -> TokenStream {
    non_fungible_data::handle_non_fungible_data(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute that derives code to encode, decode and/or describe the struct or enum, using Scrypto data and schema model.
///
/// # Example
///
/// ```ignore
/// use scrypto::prelude::*;
///
/// #[scrypto(Encode, Decode, TypeId, Describe, NonFungibleData)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn scrypto(attr: TokenStream, item: TokenStream) -> TokenStream {
    scrypto::handle_scrypto(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(item),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
