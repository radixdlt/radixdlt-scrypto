use paste::paste;
use proc_macro::TokenStream;
use quote::quote;
use radix_common::prelude::*;
use syn::{parse, spanned::Spanned, Error, Expr, Lit, Result, UnOp};

extern crate radix_common;

macro_rules! get_decimal {
    ($type:ident) => {
       paste! {
             fn [< get_ $type:snake:lower _from_expr >](expr: &Expr, negate: bool) -> Result<$type> {
                match expr {
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Str(lit_str) => {
                            // Do not allow string literal preceeded with '-', eg. -"12.3"
                            if !negate {
                                $type::try_from(lit_str.value())
                                    .map_err(|err| Error::new(lit_str.span(), [< $type:snake:lower _error_reason >](err).to_string()))
                            }
                            else {
                                Err(Error::new(
                                    lit_str.span(),
                                    "This macro only supports string, integer and float literals.",
                                ))
                            }
                        },
                        Lit::Int(lit_int) => {
                            if lit_int.suffix() != "" {
                                Err(Error::new(
                                    lit_int.span(),
                                    format!("No suffix is allowed. Remove the {}.", lit_int.suffix()),
                                ))
                            } else {
                                let mut val = $type::try_from(lit_int.base10_digits())
                                    .map_err(|err| Error::new(lit_int.span(), [< $type:snake:lower _error_reason >](err).to_string()))?;

                                // Negate received value if negate flag is set.
                                // Safe from overflow.
                                if negate {
                                    val = val.checked_neg()
                                        .ok_or(Error::new(lit_int.span(), [< $type:snake:lower _error_reason >]([< Parse $type Error >]::Overflow).to_string()))?;
                                }
                                Ok(val)
                            }
                        }
                        Lit::Float(lit_float) => {
                            if lit_float.suffix() != "" {
                                Err(Error::new(
                                    lit_float.span(),
                                    format!("No suffix is allowed. Remove the {}.", lit_float.suffix()),
                                ))
                            } else {
                                let digits = lit_float.base10_digits();

                                // Preceed the literal digits with '-" if negate flag is set.
                                // And then convert received string to decimal.
                                // This is to avoid negation of the received decimal, which
                                // overflows for MIN value.
                                let s = if negate {
                                    "-".to_string() + digits
                                } else {
                                    digits.to_string()
                                };

                                $type::try_from(s)
                                    .map_err(|err| Error::new(lit_float.span(), [< $type:snake:lower _error_reason >](err).to_string()))
                            }
                        }
                        other_lit => Err(Error::new(
                            other_lit.span(),
                            "This macro only supports string, integer and float literals.",
                        )),
                    },
                    Expr::Unary(unary) => match unary.op {
                        UnOp::Neg(unary_neg) => {
                            // Do not allow multiple '-'
                            if !negate {
                                [< get_ $type:snake:lower _from_expr >](unary.expr.as_ref(), true)
                            }
                            else {
                                Err(Error::new(
                                    unary_neg.span(),
                                    "This macro only supports string, integer and float literals.",
                                ))
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

fn decimal_error_reason(error: ParseDecimalError) -> &'static str {
    match error {
        ParseDecimalError::InvalidDigit => "There is an invalid character.",
        ParseDecimalError::Overflow => "The number is too large to fit in a decimal.",
        ParseDecimalError::EmptyIntegralPart => {
            "If there is a decimal point, the number must include at least one digit before it. Use a 0 if necessary."
        },
        ParseDecimalError::EmptyFractionalPart => {
            "If there is a decimal point, the number must include at least one digit after it."
        }
        ParseDecimalError::MoreThanEighteenDecimalPlaces => {
            "A decimal cannot have more than eighteen decimal places."
        }
        ParseDecimalError::MoreThanOneDecimalPoint => {
            "A decimal cannot have more than one decimal point."
        }
        ParseDecimalError::InvalidLength(_) => {
            unreachable!("Not a possible error from the from_str function")
        }
    }
}

fn precise_decimal_error_reason(error: ParsePreciseDecimalError) -> &'static str {
    match error {
        ParsePreciseDecimalError::InvalidDigit => "There is an invalid character",
        ParsePreciseDecimalError::Overflow => {
            "The number is too large to fit in a precise decimal."
        }
        ParsePreciseDecimalError::EmptyIntegralPart => {
            "If there is a decimal point, the number must include at least one digit before it. Use a 0 if necessary."
        }
        ParsePreciseDecimalError::EmptyFractionalPart => {
            "If there is a decimal point, the number must include at least one digit after it."
        }
        ParsePreciseDecimalError::MoreThanThirtySixDecimalPlaces => {
            "A precise decimal cannot have more than thirty-six decimal places."
        }
        ParsePreciseDecimalError::MoreThanOneDecimalPoint => {
            "A precise decimal cannot have more than one decimal point."
        }
        ParsePreciseDecimalError::InvalidLength(_) => {
            unreachable!("Not a possible error from the from_str function")
        }
    }
}

get_decimal!(Decimal);
get_decimal!(PreciseDecimal);

pub fn to_decimal(input: TokenStream) -> Result<TokenStream> {
    // Parse the input into an Expression
    let expr = parse::<Expr>(input)?;

    let decimal = get_decimal_from_expr(&expr, false)?;
    let int = decimal.attos();
    let arr = int.to_digits();
    let i0 = arr[0];
    let i1 = arr[1];
    let i2 = arr[2];

    Ok(TokenStream::from(quote! {
        radix_common::math::Decimal::from_attos(radix_common::math::I192::from_digits([#i0, #i1, #i2]))
    }))
}

pub fn to_precise_decimal(input: TokenStream) -> Result<TokenStream> {
    // Parse the input into an Expression
    let expr = parse::<Expr>(input)?;

    let decimal = get_precise_decimal_from_expr(&expr, false)?;
    let int = decimal.precise_subunits();
    let arr = int.to_digits();
    let i0 = arr[0];
    let i1 = arr[1];
    let i2 = arr[2];
    let i3 = arr[3];

    Ok(TokenStream::from(quote! {
        radix_common::math::PreciseDecimal::from_precise_subunits(radix_common::math::I256::from_digits([#i0, #i1, #i2, #i3]))
    }))
}
