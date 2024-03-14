use proc_macro::TokenStream;

#[cfg(not(panic = "unwind"))]
compile_error!("The `catch_unwind` crate requires that `panic = \"unwind\"` in the profile");

mod unwind;
use unwind::handle_catch_unwind;

mod decimal;
use decimal::{to_decimal, to_precise_decimal};

#[proc_macro_attribute]
pub fn catch_unwind(metadata: TokenStream, input: TokenStream) -> TokenStream {
    handle_catch_unwind(metadata.into(), input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn ignore(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// Creates a `Decimal` from literals.
/// It is a compile-time macro. It allows to declare constants and statics.
/// Example:
///  const D1: Decimal = dec!("1111.11111")
///  const D2: Decimal = dec!("-1111.11111")
///  const D3: Decimal = dec!(1)
///  const D4: Decimal = dec!(-1_0000_000_u128)
///
// NOTE: Decimal arithmetic operation safe unwrap.
// In general, it is assumed that reasonable literals are provided.
// If not then something is definitely wrong and panic is fine.
#[proc_macro]
pub fn dec(input: TokenStream) -> TokenStream {
    to_decimal(input).unwrap_or_else(|err| err.to_compile_error().into())
}

/// Creates a `PreciseDecimal` from literals.
/// It is a compile-time macro. It allows to declare constants and statics.
/// Example:
///  const D1: PreciseDecimal = pdec!("1111.11111")
///  const D2: PreciseDecimal = pdec!("-1111.11111")
///  const D3: PreciseDecimal = pdec!(1)
///  const D4: PreciseDecimal = pdec!(-1_0000_000_u128)
///
// NOTE: Decimal arithmetic operation safe unwrap.
// In general, it is assumed that reasonable literals are provided.
// If not then something is definitely wrong and panic is fine.
#[proc_macro]
pub fn pdec(input: TokenStream) -> TokenStream {
    to_precise_decimal(input).unwrap_or_else(|err| err.to_compile_error().into())
}
