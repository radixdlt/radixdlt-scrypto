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
/// # Motivation and Examples
///
/// Effectively it functions as a more powerful version of [paste!](https://github.com/dtolnay/paste),
/// whilst bringing the power of [quote!](https://docs.rs/quote/latest/quote/)'s variable
/// substitution to declarative macros.
///
/// This approach neatly solves the following cases:
/// 1. Wanting `paste!` to output strings or work with [attributes other than doc](https://github.com/dtolnay/paste/issues/40#issuecomment-2062953012).
/// 2. Improves readability of long procedural macros through substitution of repeated segments.
/// 3. Avoiding defining internal `macro_rules!` to handle instances where you need to do a procedural macro repeat across two conflicting expansions .
/// 4. Alternatives to [meta-variables](https://github.com/rust-lang/rust/issues/83527) such as `$count`, `$index` before
///    they are stabilized, and alternatives to some forms of append-only recursive declarative macros.
///
/// An example of case 1:
/// ```rust
/// macro_rules! impl_new_type {
///     {
///         $vis $my_type($my_inner_type)
///     } => {eager_replace!{
///         #[sbor(as_type = [!stringify! $my_inner_type])]
///         $vis struct $my_type($my_inner_type)
///
///         // ...
///     }}
/// }
/// ```
///
/// The following is an example of case 2 and case 3, which creates a much more readable macro.
/// This example is hard to do with a normal macro, because the iteration of the generics in `#ImplGenerics` and `#MyType` wouldn't be compatible with the iteration over `$trait`.
/// Instead, you have to work around it, for example with internal `macro_rules!` definitions [as per this stack overflow post](https://stackoverflow.com/a/73543948).
///
/// Using the `!SET!` functionality, we can define these token streams earlier and output them in each loop iteration.
/// This also makes the intention of the macro writer much clearer, similar to [quote!](https://docs.rs/quote/latest/quote/)
/// in procedural macros:
/// ```rust
/// macro_rules! impl_marker_traits {
///     {
///         $vis:vis $type_name:ident
///         // Arbitrary generics
///         $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
///         [
///             $($trait:ident),*
///             $(,) // Optional trailing comma
///         ]
///     } => {eager_replace!{
///         [!SET! #ImplGenerics = $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?]
///         [!SET! #TypeGenerics = $(< $( $lt ),+ >)?]
///         [!SET! #MyType = $type_name #TypeGenerics]
///
///         // Output for each marker trait
///         $(
///             impl #ImplGenerics $trait for #MyType {}
///         )*
///     }}
/// }
/// ```
///
/// An example of case 4 - a simple count function, without needing recursive macros:
/// ```rust
/// macro_rules! count_idents {
///     {
///         $($value: ident),*
///     } => {eager_replace!{
///         [!SET! #index = 0]
///         $(
///             [!SET! #ignored = $value]
///             [!SET! #index = #index + 1]
///         )*
///         #index
///     }}
/// }
/// ```
/// To quickly work this through, take `count_idents!(a, b, c)`. As a first pass, the declarative macro expands, giving:
/// ```text
/// eager_replace!{
///   [!SET! #index = 0]
///   [!SET! ignored = a]
///   [!SET! #index = #index + 1]
///   [!SET! ignored = b]
///   [!SET! #index = #index + 1]
///   [!SET! ignored = c]
///   [!SET! #index = #index + 1]
///   #index
/// }
/// ```
/// Which then evaluates by setting `#index` to `0 + 1 + 1 + 1`, and then outputting that sum.
///
/// # Details
/// ## Specific functions
///
/// * `[!concat! X Y " " Z (Hello World)]` gives `"XY Z(HelloWorld)"` by concatenating each argument without spaces, and recursing inside groups. String and char literals are first unquoted. Spaces can be added with " ".
/// * `[!ident! X Y "Z"]` gives an ident `XYZ`, using the same algorithm as `concat`.
/// * `[!literal! 31 u 32]` gives `31u32`, using the same algorithm as `concat`.
/// * `[!raw! abc #abc [!ident! test]]` outputs its contents without any nested expansion, giving `abc #abc [!ident! test]`.
/// * `[!stringify! X Y " " Z]` gives `"X Y \" \" Z"` - IMPORTANT: This uses `token_stream.into_string()` which is compiler-version dependent. Do not use if that is important. Instead, the output from `concat` should be independent of compiler version.
///
/// Note that all functions except `raw` resolve in a nested manner as you would expected, e.g.
/// ```rust
/// [!ident! X Y [!ident! Hello World] Z] // "XYHelloWorldZ"
/// ```
///
/// ## Variables
///
/// You can define variables starting with `#` which can be used outside the set call.
/// All of the following calls don't return anything, but create a variable, which can be embedded later in the macro.
///
/// * `[!SET! #MyVar = ..]` sets `#MyVar` to the given token stream.
/// * `[!SET:concat! #MyVar = ..]` sets `#MyVar` to the result of applying the `concat` function to the token stream.
/// * `[!SET:ident! #MyVar = ..]` sets `#MyVar` to the result of applying the `ident` function to the token stream.
/// * `[!SET:literal! #MyVar = ..]` sets `#MyVar` to the result of applying the `literal` function to the token stream.
/// * `[!SET:raw! #MyVar = ..]` sets `#MyVar` to the result of applying the `raw` function to the token stream.
/// * `[!SET:stringify! #MyVar = ..]` sets `#MyVar` to the result of applying the `stringify` function to the token stream.
///
/// # Future extensions
/// ## String case conversion
///
/// This could in future support case conversion like [paste](https://docs.rs/paste/latest/paste/#case-conversion).
/// e.g. `[!snakecase! ..]`, `[!camelcase! ..]`, `[!uppercase! ..]`, `[!lowercase! ..]`, `[!capitalize! ..]`, `[!decapitalize! ..]`.
/// Which all support `concat` algorithm, combined with a string function.
///
/// These can be composed to achieve things like `UPPER_SNAKE_CASE` or `lowerCamelCase`,
///
/// # Hypothetical extensions
/// None of these are likely additions, but in theory, this system could be made turing complete to decrease the amount
/// you have to reach for writing your own procedural macros.
///
/// ## Functions returning literals
/// * Integer functions like `[!sum! a b]`, `[!mod! a b]` which work on integer literal tokens.
/// * Boolean conditionals like `[!eq! a b]`, `[!lt! a b]`, `[!lte! a b]` operating on literals `[!contains! needle (haystack)]`
///
/// ## Eager expansion of macros
/// When eager expansion of macros returning literals from https://github.com/rust-lang/rust/issues/90765 is stabilized,
/// things like `[!expand_literal_macros! include!("my-poem.txt")]` will be possible.
///
/// ## Conditions and if statements
/// `[!IF! cond { .. } !ELSE! { .. }]`, for example `[!IF! [!eq! [!mod! $length 2] 0] { "even length" } !ELSE! { "odd length" }]`.
///
/// ## Labels and gotos
/// `[!LABEL:loop!]` and `[!GOTO:loop!]` would bring turing completeness - although would need a rearchitecture of the token
/// streaming logic.
#[proc_macro]
pub fn eager_replace(token_stream: TokenStream) -> TokenStream {
    eager::replace(proc_macro2::TokenStream::from(token_stream))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

const BASIC_CUSTOM_VALUE_KIND: &str = "sbor::NoCustomValueKind";
const BASIC_CUSTOM_TYPE_KIND: &str = "sbor::NoCustomTypeKind";
const BASIC_CUSTOM_SCHEMA: &str = "sbor::NoCustomSchema";

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

/// A macro for outputting tests and marker traits to assert that a type has maintained its shape over time.
///
/// There are two types of assertion modes:
/// * "fixed" mode is used to ensure that a type is unchanged.
/// * "backwards_compatible" mode is used when multiple versions of the type are permissible, but
///   newer versions of the type must be compatible with the older version where they align.
///   This mode (A) ensures that the type's current schema is equivalent to the latest version, and
///   (B) ensures that each of the schemas is a strict extension of the previous mode.
///
/// There is also a "generate" mode which can be used to export the current schema. Upon running the generated test,
/// the schema is either written to a file, or output in a panic message.
///
/// ## Initial schema generation
///
/// To output the generated schema to a file path relative to the source file, add an attribute like this -
/// and then run the test which gets generated. If using Rust Analyzer this can be run from the IDE,
/// or it can be run via `cargo test`.
///
/// ```no_run
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(generate("FILE:MyType-schema-v1.txt"))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// The test should generate the given file and then panic. If you wish to only generate the schema
/// in the panic message, you can with `#[sbor_assert(generate("INLINE"))]`.
///
/// ## Fixed schema verification
///
/// To verify the type's schema is unchanged, do:
/// ```no_run
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(fixed("FILE:MyType-schema-v1.txt"))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// Other supported options are `fixed("INLINE:<hex>")` and `fixed("CONST:<Constant>")`.
///
/// ## Backwards compatibility verification
///
/// To allow multiple backwards-compatible versions, you can do this:
/// ```no_run
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(backwards_compatible(
///     version1 = "FILE:MyType-schema-v1.txt",
///     version2 = "FILE:MyType-schema-v2.txt",
/// ))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// Instead of `"FILE:X"`, you can also use `"INLINE:<hex>"` and `"CONST:<Constant>"`.
///
/// ## Custom settings
/// By default, the `fixed` mode will use `SchemaComparisonSettings::require_equality()` and
/// the `backwards_compatible` mode will use `SchemaComparisonSettings::allow_extension()`.
///
/// You may wish to change these:
/// * If you just wish to ignore the equality of metadata such as names, you can use the
///   `allow_name_changes` flag.
/// * If you wish to override any settings, you can provide a constant containing your
///   own SchemaComparisonSettings.
///
/// For example:
/// ```no_run
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(
///     fixed("FILE:MyType-schema-v1.txt"),
///     settings(allow_name_changes),
/// )]
/// struct MyType {
///     // ...
/// }
///
/// const CUSTOM_COMPARISON_SETTINGS: sbor::schema::SchemaComparisonSettings = sbor::schema::SchemaComparisonSettings::require_equality();
///
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(
///     backwards_compatible(
///         version1 = "FILE:MyType-schema-v1.txt",
///         version2 = "FILE:MyType-schema-v2.txt",
///     ),
///     settings(CUSTOM_COMPARISON_SETTINGS),
/// )]
/// struct MyOtherType {
///     // ...
/// }
/// ```
#[proc_macro_derive(BasicSborAssertion, attributes(sbor_assert))]
pub fn basic_sbor_assertion(input: TokenStream) -> TokenStream {
    sbor_derive_common::sbor_assert::handle_sbor_assert_derive(
        proc_macro2::TokenStream::from(input),
        BASIC_CUSTOM_SCHEMA,
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
