use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::Parser;
use syn::{punctuated::Punctuated, token::Comma, *};

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_scrypto(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    trace!("handle_scrypto() starts");

    let DeriveInput {
        ident,
        data,
        attrs: _,
        vis,
        generics,
    } = parse2(item)?;

    let parser = Punctuated::<Ident, Comma>::parse_terminated;
    let idents = parser.parse2(attr)?;
    let mut derived_attributes = Vec::<Attribute>::new();
    let mut sbor = false;
    for ident in idents {
        match ident.to_string().as_str() {
            "Encode" => {
                sbor = true;
                derived_attributes.push(parse_quote! {
                    #[derive(::sbor::Encode)]
                })
            }
            "Decode" => {
                sbor = true;
                derived_attributes.push(parse_quote! {
                    #[derive(::sbor::Decode)]
                })
            }
            "TypeId" => {
                sbor = true;
                derived_attributes.push(parse_quote! {
                    #[derive(::sbor::TypeId)]
                })
            }
            "Describe" => derived_attributes.push(parse_quote! {
                #[derive(radix_engine_derive::Describe)]
            }),
            "NonFungibleData" => derived_attributes.push(parse_quote! {
                #[derive(::scrypto::NonFungibleData)]
            }),
            _ => {}
        }
    }
    if sbor {
        derived_attributes.push(parse_quote! {
            #[sbor(custom_type_id = "radix_engine_interface::data::ScryptoCustomTypeId")]
        })
    }

    let output = match &data {
        Data::Struct(DataStruct {
            struct_token,
            fields,
            semi_token,
        }) => quote! {
            #(#derived_attributes)*
            #vis #struct_token #ident #generics #fields #semi_token
        },
        Data::Enum(DataEnum {
            enum_token,
            brace_token: _,
            variants,
        }) => quote! {
            #(#derived_attributes)*
            #vis #enum_token #ident #generics { #variants }
        },
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("ScryptoData", &output);

    trace!("handle_scrypto() finishes");
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
    fn test_scrypto_data_with_struct() {
        let attr =
            TokenStream::from_str("Encode, Decode, TypeId, Describe, NonFungibleData").unwrap();
        let item = TokenStream::from_str(
            "pub struct MyStruct<T: Bound> { pub field_1: T, pub field_2: String, }",
        )
        .unwrap();
        let output = handle_scrypto(attr, item).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::Encode)]
                #[derive(::sbor::Decode)]
                #[derive(::sbor::TypeId)]
                #[derive(::radix_engine_derive::Describe)]
                #[derive(::scrypto::NonFungibleData)]
                #[sbor(custom_type_id = "::scrypto::data::ScryptoCustomTypeId")]
                pub struct MyStruct<T: Bound> {
                    pub field_1: T,
                    pub field_2: String,
                }
            },
        );
    }

    #[test]
    fn test_scrypto_data_with_enum() {
        let attr =
            TokenStream::from_str("Encode, Decode, TypeId, Describe, NonFungibleData").unwrap();
        let item = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_scrypto(attr, item).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::Encode)]
                #[derive(::sbor::Decode)]
                #[derive(::sbor::TypeId)]
                #[derive(::radix_engine_derive::Describe)]
                #[derive(::scrypto::NonFungibleData)]
                #[sbor(custom_type_id = "::scrypto::data::ScryptoCustomTypeId")]
                enum MyEnum<T: Bound> {
                    A { named: T },
                    B(String),
                    C
                }
            },
        );
    }
}
