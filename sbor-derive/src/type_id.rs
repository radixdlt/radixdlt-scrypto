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

pub fn handle_type_id(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_type_id() starts");

    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parse2(input)?;
    let (impl_generics, ty_generics, where_clause, sbor_cti) =
        build_custom_type_id_generic(&generics, &attrs)?;

    let output = match data {
        Data::Struct(_) => quote! {
            impl #impl_generics ::sbor::TypeId <#sbor_cti> for #ident #ty_generics #where_clause {
                #[inline]
                fn type_id() -> ::sbor::SborTypeId <#sbor_cti> {
                    ::sbor::SborTypeId::Tuple
                }
            }
        },
        Data::Enum(_) => quote! {
            impl #impl_generics ::sbor::TypeId <#sbor_cti> for #ident #ty_generics #where_clause {
                #[inline]
                fn type_id() -> ::sbor::SborTypeId <#sbor_cti> {
                    ::sbor::SborTypeId::Enum
                }
            }
        },
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("TypeId", &output);

    trace!("handle_type_id() finishes");
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
        let output = handle_type_id(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomTypeId> ::sbor::TypeId<X> for Test {
                    #[inline]
                    fn type_id() -> ::sbor::SborTypeId<X> {
                        ::sbor::SborTypeId::Tuple
                    }
                }
            },
        );
    }

    #[test]
    fn test_type_id_struct_generics() {
        let input = TokenStream::from_str("struct Test<A> {a: A}").unwrap();
        let output = handle_type_id(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <A, X: ::sbor::CustomTypeId> ::sbor::TypeId<X> for Test<A> {
                    #[inline]
                    fn type_id() -> ::sbor::SborTypeId<X> {
                        ::sbor::SborTypeId::Tuple
                    }
                }
            },
        );
    }

    #[test]
    fn test_type_id_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_type_id(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomTypeId> ::sbor::TypeId<X> for Test {
                    #[inline]
                    fn type_id() -> ::sbor::SborTypeId<X> {
                        ::sbor::SborTypeId::Enum
                    }
                }
            },
        );
    }
}
