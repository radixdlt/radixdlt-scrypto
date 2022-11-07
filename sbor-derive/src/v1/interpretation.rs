use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_interpretation(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_interpretation() starts");

    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse2(input).expect("Unable to parse input");
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    trace!("Encoding: {}", ident);

    let output = match data {
        Data::Struct(_) => quote! {
            impl #impl_generics ::sbor::v1::Interpretation for #ident #ty_generics #where_clause {
                const INTERPRETATION: u8 = ::sbor::v1::DefaultInterpretations::STRUCT;
            }
        },
        Data::Enum(_) => quote! {
            impl #impl_generics ::sbor::v1::Interpretation for #ident #ty_generics #where_clause {
                const INTERPRETATION: u8 = ::sbor::v1::DefaultInterpretations::ENUM;
            }
        },
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };
    trace!("handle_interpretation() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Interpretation", &output);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_type_id_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_interpretation(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::v1::Interpretation for Test {
                    const INTERPRETATION: u8 = ::sbor::v1::DefaultInterpretations::STRUCT;
                }
            },
        );
    }

    #[test]
    fn test_type_id_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_interpretation(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::v1::Interpretation for Test {
                    const INTERPRETATION: u8 = ::sbor::v1::DefaultInterpretations::ENUM;
                }
            },
        );
    }
}
