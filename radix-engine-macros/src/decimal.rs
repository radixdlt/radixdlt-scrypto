use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

extern crate radix_engine_common;

fn get_decimal_from_expr(expr: &Expr) -> radix_engine_common::math::Decimal {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(lit_str) => {
                radix_engine_common::math::Decimal::try_from(lit_str.value()).unwrap()
            }
            syn::Lit::Int(lit_int) => {
                radix_engine_common::math::Decimal::try_from(lit_int.base10_digits()).unwrap()
            }
            syn::Lit::Bool(lit_bool) => radix_engine_common::math::Decimal::from(lit_bool.value),
            _ => panic!("Unsupported literal type!"),
        },
        Expr::Group(group) => get_decimal_from_expr(&group.expr),
        Expr::Unary(unary) => match unary.op {
            syn::UnOp::Neg(_) => -get_decimal_from_expr(unary.expr.as_ref()),
            _ => panic!("Unsupported unary expression!"),
        },
        _ => panic!("Unsupported expression!"),
    }
}

fn get_precise_decimal_from_expr(expr: &Expr) -> radix_engine_common::math::PreciseDecimal {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(lit_str) => {
                radix_engine_common::math::PreciseDecimal::try_from(lit_str.value()).unwrap()
            }
            syn::Lit::Int(lit_int) => {
                radix_engine_common::math::PreciseDecimal::try_from(lit_int.base10_digits())
                    .unwrap()
            }
            syn::Lit::Bool(lit_bool) => {
                radix_engine_common::math::PreciseDecimal::from(lit_bool.value)
            }
            _ => panic!("Unsupported literal type!"),
        },
        Expr::Group(group) => get_precise_decimal_from_expr(&group.expr),
        Expr::Unary(unary) => match unary.op {
            syn::UnOp::Neg(_) => -get_precise_decimal_from_expr(unary.expr.as_ref()),
            _ => panic!("Unsupported unary expression!"),
        },
        _ => panic!("Unsupported expression!"),
    }
}

pub fn to_decimal(input: TokenStream) -> TokenStream {
    // Parse the input into an Expression
    let expr = parse_macro_input!(input as Expr);

    let decimal = get_decimal_from_expr(&expr);
    let int = decimal.0;
    let arr = int.to_digits();
    let i0 = arr[0];
    let i1 = arr[1];
    let i2 = arr[2];

    TokenStream::from(quote! {
        radix_engine_common::math::Decimal(radix_engine_common::math::I192::from_digits([#i0, #i1, #i2]))
    })
}

pub fn to_precise_decimal(input: TokenStream) -> TokenStream {
    // Parse the input into an Expression
    let expr = parse_macro_input!(input as Expr);

    let decimal = get_precise_decimal_from_expr(&expr);
    let int = decimal.0;
    let arr = int.to_digits();
    let i0 = arr[0];
    let i1 = arr[1];
    let i2 = arr[2];
    let i3 = arr[2];

    TokenStream::from(quote! {
        radix_engine_common::math::PreciseDecimal(radix_engine_common::math::I256::from_digits([#i0, #i1, #i2, #i3]))
    })
}
