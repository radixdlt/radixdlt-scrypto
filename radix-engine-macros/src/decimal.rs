use paste::paste;
use proc_macro::TokenStream;
use quote::quote;
use radix_engine_common::prelude::CheckedNeg;
use syn::{parse, spanned::Spanned, Error, Expr, Lit, Result, UnOp};

extern crate radix_engine_common;
use radix_engine_common::math::{Decimal, PreciseDecimal};

macro_rules! get_decimal {
    ($type:ident) => {
       paste! {
             fn [< get_ $type:snake:lower _from_expr >](expr: &Expr) -> Result<$type> {
                match expr {
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Str(lit_str) => $type::try_from(lit_str.value())
                            .map_err(|err| Error::new(lit_str.span(), format!("Parsing failed due to {:?}", err))),
                        Lit::Int(lit_int) => {
                            if lit_int.suffix() != "" {
                                Err(Error::new(
                                    lit_int.span(),
                                    format!("No suffix is allowed. Remove the {}.", lit_int.suffix()),
                                ))
                            } else {
                                $type::try_from(lit_int.base10_digits())
                                    .map_err(|err| Error::new(lit_int.span(), format!("Parsing failed due to {:?}", err)))
                            }
                        }
                        Lit::Float(lit_float) => {
                            if lit_float.suffix() != "" {
                                Err(Error::new(
                                    lit_float.span(),
                                    format!("No suffix is allowed. Remove the {}.", lit_float.suffix()),
                                ))
                            } else {
                                $type::try_from(lit_float.base10_digits())
                                    .map_err(|err| Error::new(lit_float.span(), format!("Parsing failed due to {:?}", err)))
                            }
                        }
                        other_lit => Err(Error::new(
                            other_lit.span(),
                            "This macro only supports string, integer and float literals.",
                        )),
                    },
                    Expr::Unary(unary) => match unary.op {
                        UnOp::Neg(unary_neg) => {
                            let res = [< get_ $type:snake:lower _from_expr >](unary.expr.as_ref());
                            match res {
                                Ok(val) => {
                                    let val = val.checked_neg().ok_or(Error::new(unary_neg.span, "Parsing failed due to Overflow"))?;
                                    Ok(val)
                                },
                                Err(err) => Err(Error::new(unary_neg.span, err)),
                            }
                        }
                        other_unary => Err(Error::new(
                            other_unary.span(),
                            "This macro only supports string, integer and float literals.",
                        )),
                    },
                    other_expr => Err(Error::new(
                        other_expr.span(),
                        "This macro only supports string, integer and float literals.",
                    )),
                }
            }

        }
    };
}

get_decimal!(Decimal);
get_decimal!(PreciseDecimal);

pub fn to_decimal(input: TokenStream) -> Result<TokenStream> {
    // Parse the input into an Expression
    let expr = parse::<Expr>(input)?;

    let decimal = get_decimal_from_expr(&expr)?;
    let int = decimal.0;
    let arr = int.to_digits();
    let i0 = arr[0];
    let i1 = arr[1];
    let i2 = arr[2];

    Ok(TokenStream::from(quote! {
        radix_engine_common::math::Decimal(radix_engine_common::math::I192::from_digits([#i0, #i1, #i2]))
    }))
}

pub fn to_precise_decimal(input: TokenStream) -> Result<TokenStream> {
    // Parse the input into an Expression
    let expr = parse::<Expr>(input)?;

    let decimal = get_precise_decimal_from_expr(&expr)?;
    let int = decimal.0;
    let arr = int.to_digits();
    let i0 = arr[0];
    let i1 = arr[1];
    let i2 = arr[2];
    let i3 = arr[3];

    Ok(TokenStream::from(quote! {
        radix_engine_common::math::PreciseDecimal(radix_engine_common::math::I256::from_digits([#i0, #i1, #i2, #i3]))
    }))
}
