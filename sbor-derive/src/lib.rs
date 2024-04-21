use proc_macro::TokenStream;
use std::str::FromStr;
mod eager;

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

/// NOTE: This should probably be moved out of sbor to its own crate.
///
/// This macro causes the `eager_stringify!` pseudo-macro to stringify its contents immediately.
/// Similar to the `paste!` macro, this is intended for use in declarative macros.
///
/// It is particularly useful in scenarios where `paste` doesn't work - in particular, to
/// create non-idents, or to create non-doc attribute string content, which paste cannot do, e.g.:
/// ```rust
/// // Inside a macro_rules! expression:
/// eager_replace!{
///     #[sbor(as_type = eager_stringify!($my_inner_type))]
///     $vis struct $my_type($my_inner_type)
/// }
/// ```
///
/// ## Use with docs
///
/// You can combine `eager_stringify!` with the `paste` macro's ability to concat doc string literals together,
/// as follows. In some cases, `paste` can be used without `eager_stringify!` for the same effect.
/// ```rust
/// // Inside a macro_rules! expression:
/// eager_replace!{paste!{
///     #[doc = "This is the [`" eager_stringify!($my_type $(< $( $generic_type ),+ >)?) "`] type."]
///     $vis struct $my_type $(< $( $generic_type ),+ >)?(
///         $my_inner_type $(< $( $generic_type ),+ >)?
///     )
/// }}
/// ```
///
/// ## Future vision
///
/// The below describes a future vision which would expand this macro into a powerful
/// but simple general-purpose tool to ease building declarative macros.
///
/// Effectively it would be a more powerful version of [paste](https://github.com/dtolnay/paste)
/// whilst bringing the power of [quote](https://docs.rs/quote/latest/quote/)'s variable
/// substitution to declarative macros.
///
/// This approach neatly solves the following cases:
/// * [Pasting into non-doc attributes](https://github.com/dtolnay/paste/issues/40#issuecomment-2062953012)
/// * Simplify handling sub-repetition which currently needs an internal `macro_rules!` definition [per this stack overflow post](https://stackoverflow.com/a/73543948)
/// * Improved readability of long procedural macros through substitution of repeated segments
///
/// ### More types
///
/// Output `string`, `ident`, `literal` or just a token stream:
/// * `[!EAGER!string](X Y " " Z)` gives "XY Z" concats each argument stringified without spaces
///   (except removing the quotes around string literals). Spaces can be added with " ".
/// * `[!EAGER!ident]` does the same for idents
/// * `[!EAGER!literal]` does the same for literals
/// * `[!EAGER!]` parses and outputs a token stream.
///     This would be a no-op, except it's not when combined with other features below:
///     variables, nested calls, etc.
///
/// ### Variables + Cleaner Coding
///
/// You can define variables starting with `#` which can be used inside other eager evaluations.
///
/// The command `[!EAGER!define:#MyZ:ident](ZZZ)` doesn't output anything, but sets `#MyZ`
/// to be the given `Ident`. Then, insides any other eager macro, `#MyZ` outputs the given ident.
///
/// This would also work for literals, strings and token streams.
///
/// ### Nesting
///
/// Would add support for nesting [!EAGER!] calls inside eachother - although
/// using variables might well be cleaner code.
///
/// ### String case conversion
///
/// Could in future support case conversion like [paste](https://docs.rs/paste/latest/paste/#case-conversion).
///
/// ### Alternative EAGER tag
///
/// Would allow a user to specify their own tag instead of `EAGER`. This could:
/// * Facilitate nesting `eager_replace` calls: `eager_replace!{ [!tag = INNER_EAGER] code...}`
/// * Allow using shorthand e.g. `E` instead
///
/// ### Example of future vision
/// ```rust
/// macro_rules! impl_marker_traits {
///     {
///         $(#[$attributes:meta])*
///         $vis:vis $type_name:ident
///         // Arbitrary generics
///         $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
///         [
///             $($trait:ident),*
///             $(,) // Optional trailing comma
///         ]
///     } => {eager_replace!{
///         [!EAGER!define:#ImplGenerics]($(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?)
///         [!EAGER!define:#TypeGenerics]($(< $( $lt ),+ >)?)
///
///         // Output for each marker trait
///         $(
///             // NOTE: [!EAGER] outputs a token stream, not just a token tree
///             //       so it can be used for outputting things like
///             //       enum variants and attributes where a declarative macro
///             //       couldn't be used
///             [!EAGER!]{ impl #ImplGenerics $trait for Type #TypeGenerics }
///             {
///                 // Empty trait body
///             }
///         )*
///     }}
/// }
/// ```
#[proc_macro]
pub fn eager_replace(token_stream: TokenStream) -> TokenStream {
    eager::replace_recursive(token_stream)
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
