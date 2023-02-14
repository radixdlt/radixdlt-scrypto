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

    let parsed: DeriveInput = parse2(input)?;
    let is_transparent = is_transparent(&parsed.attrs);

    let output = if is_transparent {
        handle_transparent_categorize(parsed, context_custom_value_kind)?
    } else {
        handle_normal_categorize(parsed, context_custom_value_kind)?
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Categorize", &output);

    trace!("handle_categorize() finishes");
    Ok(output)
}

fn handle_normal_categorize(
    parsed: DeriveInput,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parsed;
    let (impl_generics, ty_generics, where_clause, sbor_cvk) =
        build_custom_categorize_generic(&generics, &attrs, context_custom_value_kind, false)?;

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

    Ok(output)
}

fn handle_transparent_categorize(
    parsed: DeriveInput,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parsed;
    let (impl_generics, ty_generics, where_clause, sbor_cvk) =
        build_custom_categorize_generic(&generics, &attrs, context_custom_value_kind, true)?;
    let output = match data {
        Data::Struct(s) => {
            let FieldsData {
                unskipped_field_types,
                ..
            } = process_fields_for_categorize(&s.fields);
            if unskipped_field_types.len() != 1 {
                return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
            }
            let field_type = &unskipped_field_types[0];

            quote! {
                impl #impl_generics ::sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind <#sbor_cvk> {
                        <#field_type as ::sbor::Categorize::<#sbor_cvk>>::value_kind()
                    }
                }
            }
        }
        Data::Enum(_) => {
            return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

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
    fn test_categorize_transparent_struct() {
        let input =
            TokenStream::from_str("#[sbor(transparent)] struct Test {a: u32, #[sbor(skip)]b: u16}")
                .unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        <u32 as ::sbor::Categorize::<X>>::value_kind()
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
    fn test_categorize_transparent_struct_generics() {
        let input = TokenStream::from_str("#[sbor(transparent)] struct Test<A> {a: A}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <A: ::sbor::Categorize<X>, X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test<A> {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        <A as ::sbor::Categorize::<X>>::value_kind()
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
