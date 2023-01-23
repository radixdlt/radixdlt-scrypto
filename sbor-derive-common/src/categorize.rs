use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_categorize(
    input: TokenStream,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    trace!("handle_categorize() starts");

    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parse2(input)?;
    let (impl_generics, ty_generics, where_clause, sbor_cvk) =
        build_custom_categorize_generic(&generics, &attrs, context_custom_value_kind)?;

    let output = match data {
        Data::Struct(_) => quote! {
            impl #impl_generics ::sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                #[inline]
                fn value_kind() -> ::sbor::ValueKind <#sbor_cvk> {
                    ::sbor::ValueKind::Tuple
                }
            }
        },
        Data::Enum(_) => quote! {
            impl #impl_generics ::sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                #[inline]
                fn value_kind() -> ::sbor::ValueKind <#sbor_cvk> {
                    ::sbor::ValueKind::Enum
                }
            }
        },
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Categorize", &output);

    trace!("handle_categorize() finishes");
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
    fn test_categorize_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        ::sbor::ValueKind::Tuple
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_struct_generics() {
        let input = TokenStream::from_str("struct Test<A> {a: A}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <A, X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test<A> {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        ::sbor::ValueKind::Tuple
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        ::sbor::ValueKind::Enum
                    }
                }
            },
        );
    }
}
