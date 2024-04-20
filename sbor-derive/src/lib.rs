use proc_macro::TokenStream;
use std::str::FromStr;
mod eager_stringify;

/// Derive code that returns the value kind.
#[proc_macro_derive(Categorize, attributes(sbor))]
pub fn categorize(input: TokenStream) -> TokenStream {
    sbor_derive_common::categorize::handle_categorize(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that encodes this data structure
#[proc_macro_derive(Encode, attributes(sbor))]
pub fn encode(input: TokenStream) -> TokenStream {
    sbor_derive_common::encode::handle_encode(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that decodes this data structure from a byte array.
#[proc_macro_derive(Decode, attributes(sbor))]
pub fn decode(input: TokenStream) -> TokenStream {
    sbor_derive_common::decode::handle_decode(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that describes this type.
#[proc_macro_derive(Describe, attributes(sbor))]
pub fn describe(input: TokenStream) -> TokenStream {
    sbor_derive_common::describe::handle_describe(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// A shortcut for [`Categorize`], [`Encode`], [`Decode`], and [`Describe`] derives.
///
#[proc_macro_derive(Sbor, attributes(sbor))]
pub fn sbor(input: TokenStream) -> TokenStream {
    sbor_derive_common::sbor::handle_sbor(proc_macro2::TokenStream::from(input), None, None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// An empty derive which exists solely to allow the helper "sbor" attribute
/// to be used without generating a compile error.
///
/// The intended use-case is as a utility for building other macros,
/// which wish to add sbor attribute annotations to types for when they do
/// use an Sbor derive - but wish to avoid the following error when they don't:
/// ```text
/// error: cannot find attribute `sbor` in this scope
/// ```
///
/// Ideally this would output an empty token stream, but instead we
/// return a simply comment, to avoid the proc macro system thinking
/// the macro build has broken and returning this error:
/// ```text
/// proc macro `PermitSborAttributes` not expanded: internal error
/// ```
#[proc_macro_derive(PermitSborAttributes, attributes(sbor))]
pub fn permit_sbor_attributes(_: TokenStream) -> TokenStream {
    TokenStream::from_str(&"// Empty PermitSborAttributes expansion").unwrap()
}

/// Allows the `eager_stringify!` macro to stringify its contents immediately.
#[proc_macro]
pub fn enable_eager_stringify(token_stream: TokenStream) -> TokenStream {
    eager_stringify::replace_recursive(token_stream)
}

const BASIC_CUSTOM_VALUE_KIND: &str = "sbor::NoCustomValueKind";
const BASIC_CUSTOM_TYPE_KIND: &str = "sbor::NoCustomTypeKind";

/// Derive code that returns the value kind - specifically for Basic SBOR.
#[proc_macro_derive(BasicCategorize, attributes(sbor))]
pub fn basic_categorize(input: TokenStream) -> TokenStream {
    sbor_derive_common::categorize::handle_categorize(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Derive code that encodes this data structure - specifically for Basic SBOR.
#[proc_macro_derive(BasicEncode, attributes(sbor))]
pub fn basic_encode(input: TokenStream) -> TokenStream {
    sbor_derive_common::encode::handle_encode(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Derive code that decodes this data structure from a byte array - specifically for Basic SBOR.
#[proc_macro_derive(BasicDecode, attributes(sbor))]
pub fn basic_decode(input: TokenStream) -> TokenStream {
    sbor_derive_common::decode::handle_decode(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Derive code that describes the type - specifically for Basic SBOR.
#[proc_macro_derive(BasicDescribe, attributes(sbor))]
pub fn basic_describe(input: TokenStream) -> TokenStream {
    sbor_derive_common::describe::handle_describe(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_TYPE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// A shortcut for [`BasicCategorize`], [`BasicEncode`], [`BasicDecode`], and [`BasicDescribe`] derives.
///
#[proc_macro_derive(BasicSbor, attributes(sbor))]
pub fn basic_sbor(input: TokenStream) -> TokenStream {
    sbor_derive_common::sbor::handle_sbor(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
        Some(BASIC_CUSTOM_TYPE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
