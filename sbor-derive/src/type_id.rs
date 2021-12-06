use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_type_id(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_type_id() starts");

    let DeriveInput { ident, data, .. } = parse2(input).expect("Unable to parse input");
    trace!("Encoding: {}", ident);

    let output = match data {
        Data::Struct(_) => quote! {
            impl ::sbor::TypeId for #ident {
                #[inline]
                fn type_id() -> u8 {
                    ::sbor::type_id::TYPE_STRUCT
                }
            }
        },
        Data::Enum(_) => quote! {
            impl ::sbor::TypeId for #ident {
                #[inline]
                fn type_id() -> u8 {
                    ::sbor::type_id::TYPE_ENUM
                }
            }
        },
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };
    trace!("handle_type_id() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("TypeId", &output);

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
                impl ::sbor::TypeId for Test {
                    #[inline]
                    fn type_id() -> u8 {
                        ::sbor::type_id::TYPE_STRUCT
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
                impl ::sbor::TypeId for Test {
                    #[inline]
                    fn type_id() -> u8 {
                        ::sbor::type_id::TYPE_ENUM
                    }
                }
            },
        );
    }
}
