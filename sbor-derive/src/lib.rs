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
/// This macro is a powerful but simple general-purpose tool to ease building declarative macros which create
/// new types.
///
/// ## Motivation
///
/// Effectively it functions as a more powerful version of [paste!](https://github.com/dtolnay/paste),
/// whilst bringing the power of [quote!](https://docs.rs/quote/latest/quote/)'s variable
/// substitution to declarative macros.
///
/// This approach neatly solves the following cases:
/// 1. Wanting `paste!` to output strings or work with [attributes other than doc](https://github.com/dtolnay/paste/issues/40#issuecomment-2062953012).
/// 2. Avoiding defining internal `macro_rules!` to handle instances where you need to do a procedural macro repeat across two conflicting expansions [per this stack overflow post](https://stackoverflow.com/a/73543948).
/// 3. Improves readability of long procedural macros through substitution of repeated segments.
///
/// An example of case 1:
/// ```rust
/// // Inside a macro_rules! expression:
/// eager_replace!{
///     #[sbor(as_type = [!stringify! $my_inner_type])]
///     $vis struct $my_type($my_inner_type)
/// }
/// ```
///
/// ## Specific functions
///
/// * `[!stringify! X Y " " Z]` gives `"X Y \" \" Z"` - IMPORTANT: This uses `token_stream.into_string()` which is compiler-version dependent. Do not use if that is important. Instead, the output from `concat` should be independent of compiler version.
/// * `[!concat! X Y " " Z (Hello World)]` gives `"XY Z(HelloWorld)"` by concatenating each argument without spaces, and recursing inside groups. String and char literals are first unquoted. Spaces can be added with " ".
/// * `[!ident! X Y "Z"]` gives an ident `XYZ`, using the same algorithm as `concat`.
/// * `[!literal! 31 u 32]` gives `31u32`, using the same algorithm as `concat`.
/// * `[!raw! abc #abc [!ident! test]]` outputs its contents without any nested expansion, giving `abc #abc [!ident! test]`.
///
/// Note that all functions except `raw` resolve in a nested manner as you would expected, e.g.
/// ```rust
/// [!concat! X Y [!ident! Hello World] Z] // "XYHelloWorldZ"
/// ```
///
/// ## Variables for cleaner coding
///
/// You can define variables starting with `#` which can be used outside the set call.
/// All of the following calls don't return anything, but create a variable, which can be embedded later in the macro.
/// See the `Demonstration` section for details
///
/// * `[!SET! #MyVar = ..]` sets `#MyVar` to the given token stream.
/// * `[!SET:stringify! #MyVar = ..]` sets `#MyVar` to the result of applying the `stringify` function to the token stream.
/// * `[!SET:concat! #MyVar = ..]` sets `#MyVar` to the result of applying the `concat` function to the token stream.
/// * `[!SET:ident! #MyVar = ..]` sets `#MyVar` to the result of applying the `ident` function to the token stream.
/// * `[!SET:literal! #MyVar = ..]` sets `#MyVar` to the result of applying the `literal` function to the token stream.
///
/// ## Demonstration
/// ```rust
/// macro_rules! impl_marker_traits {
///     {
///         $(#[$attributes:meta])*
///         $vis:vis $type_name_suffix:ident
///         // Arbitrary generics
///         $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
///         [
///             $($trait:ident),*
///             $(,) // Optional trailing comma
///         ]
///     } => {eager_replace!{
///         [!SET! #ImplGenerics = $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?]
///         [!SET! #TypeGenerics = $(< $( $lt ),+ >)?]
///         [!SET:ident! #MyType = Type $type_name_suffix #TypeGenerics]
///
///         // Output for each marker trait
///         $(
///             impl #ImplGenerics $trait for #MyType
///             {
///                 // Empty trait body
///             }
///         )*
///     }}
/// }
/// ```
///
/// ## Future extensions
/// ### String case conversion
///
/// Could in future support case conversion like [paste](https://docs.rs/paste/latest/paste/#case-conversion).
#[proc_macro]
pub fn eager_replace(token_stream: TokenStream) -> TokenStream {
    eager::replace(proc_macro2::TokenStream::from(token_stream))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
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
