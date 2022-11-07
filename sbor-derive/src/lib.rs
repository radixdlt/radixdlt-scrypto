mod decode;
mod describe;
mod encode;
mod type_id;
mod utils;

mod v1;

use proc_macro::TokenStream;

/// Derive code that describes this data structure.
///
/// Note that this derive doesn't work with recursive type, such as
/// ```ignore
/// struct A {
///     array: Vec<A>
/// }
/// ```
#[proc_macro_derive(Describe, attributes(sbor))]
pub fn describe(input: TokenStream) -> TokenStream {
    describe::handle_describe(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that returns the type ID.
#[proc_macro_derive(TypeId, attributes(sbor))]
pub fn type_id(input: TokenStream) -> TokenStream {
    type_id::handle_type_id(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that encodes this data structure
#[proc_macro_derive(Encode, attributes(sbor))]
pub fn encode(input: TokenStream) -> TokenStream {
    encode::handle_encode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that decodes this data structure from a byte array.
#[proc_macro_derive(Decode, attributes(sbor))]
pub fn decode(input: TokenStream) -> TokenStream {
    decode::handle_decode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that decodes this data structure from a byte array.
#[proc_macro_derive(V1Interpretation, attributes(sbor))]
pub fn v1_interpretation(input: TokenStream) -> TokenStream {
    v1::interpretation::handle_interpretation(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(V1Encode, attributes(sbor))]
pub fn v1_encode(input: TokenStream) -> TokenStream {
    v1::encode::handle_encode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(V1Decode, attributes(sbor))]
pub fn v1_decode(input: TokenStream) -> TokenStream {
    v1::decode::handle_decode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(V1Schema, attributes(sbor))]
pub fn v1_schema(input: TokenStream) -> TokenStream {
    v1::schema::handle_schema(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
