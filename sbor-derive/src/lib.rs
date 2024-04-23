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
/// This macro is a powerful but simple general-purpose tool to ease building declarative macros.
///
/// Effectively it functions as a more powerful version of [paste](https://github.com/dtolnay/paste),
/// whilst bringing the power of [quote](https://docs.rs/quote/latest/quote/)'s variable
/// substitution to declarative macros.
///
/// This approach neatly solves the following cases:
/// * [Pasting into non-doc attributes](https://github.com/dtolnay/paste/issues/40#issuecomment-2062953012)
/// * Simplify handling sub-repetition which currently needs an internal `macro_rules!` definition [per this stack overflow post](https://stackoverflow.com/a/73543948)
/// * Improved readability of long procedural macros through substitution of repeated segments
///
///
/// It is particularly useful in scenarios where `paste` doesn't work - in particular, to
/// create non-idents, or to create non-doc attribute string content, which paste cannot do, e.g.:
/// ```rust
/// // Inside a macro_rules! expression:
/// eager_replace!{
///     #[sbor(as_type = [!EAGER:stringify! $my_inner_type])]
///     $vis struct $my_type($my_inner_type)
/// }
/// ```
///
/// ## Specific functions
///
/// * `[!EAGER:stringify! X Y " " Z]` gives `"XY \" \" Z"`
/// * `[!EAGER:concat! X Y " " Z]` gives `"XY Z"` by concatenating each argument stringified without spaces. String and Char literals are first unquoted. Spaces can be added with " ".
/// * `[!EAGER:ident! X Y "Z"]` gives an ident `XYZ`.
/// * `[!EAGER:literal! 31 u 32]` gives `31u32`.
/// * `[!EAGER! ...]` outputs the `...` token stream, can be used for outputting `#[!EAGER! ident]` so that `#ident` isn't detected as a variable.
///
/// ## Variables for cleaner coding
///
/// You can define variables starting with `#` which can be used outside the set call.
///
/// * The command `[!EAGER:set! #MyZ = 1 + 2]` doesn't output anything, but sets `#MyZ` to the given token stream.
/// * Similarly `[!EAGER:set:ident! #MyZ = ZZZ]` sets `#MyZ` as an ident. This also works with `stringify`, `concat` and `literal`.
///
/// ### Demonstration
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
///         [!EAGER:set! #ImplGenerics = $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?]
///         [!EAGER:set! #TypeGenerics = $(< $( $lt ),+ >)?]
///         [!EAGER:set:ident! #MyType = Type $type_name_suffix #TypeGenerics]
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
