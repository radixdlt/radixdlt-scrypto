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

    let parser = Punctuated::<Path, Comma>::parse_terminated;
    let paths = parser.parse2(attr)?;
    let mut derived_attributes = Vec::<Attribute>::new();
    let mut add_custom_value_kind = false;
    for path in paths {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect();

        // best-effort sbor detection
        match segments.join("::").as_str() {
            "Categorize" | "Encode" | "Decode" | "sbor::Categorize" | "sbor::Encode"
            | "sbor::Decode" => add_custom_value_kind = true,
            _ => {}
        }

        derived_attributes.push(parse_quote! {
          #[derive(#path)]
        });
    }
    if add_custom_value_kind {
        derived_attributes.push(parse_quote! {
            #[sbor(custom_value_kind = "radix_engine_interface::data::ScryptoCustomValueKind")]
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
    fn test_variant_paths() {
        let attr =
            TokenStream::from_str("Encode, Debug, std::fmt::Debug, ::std::fmt::Debug").unwrap();
        let item = TokenStream::from_str("pub struct MyStruct { }").unwrap();
        let output = handle_scrypto(attr, item).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(Encode)]
                #[derive(Debug)]
                #[derive(std::fmt::Debug)]
                #[derive(::std::fmt::Debug)]
                #[sbor(custom_value_kind = "radix_engine_interface::data::ScryptoCustomValueKind")]
                pub struct MyStruct {
                }
            },
        );
    }

    #[test]
    fn test_full_paths() {
        for s in [
            "Categorize",
            "Encode",
            "Decode",
            "sbor::Categorize",
            "sbor::Encode",
            "sbor::Decode",
            "::sbor::Categorize",
            "::sbor::Encode",
            "::sbor::Decode",
        ] {
            let attr = TokenStream::from_str(s).unwrap();
            let item = TokenStream::from_str("pub struct MyStruct { }").unwrap();
            let output = handle_scrypto(attr, item).unwrap();

            assert!(output.to_string().contains("ScryptoCustomValueKind"));
        }
    }

    #[test]
    fn test_scrypto_data_with_struct() {
        let attr =
            TokenStream::from_str("Encode, Decode, Categorize, Describe, NonFungibleData").unwrap();
        let item = TokenStream::from_str(
            "pub struct MyStruct<T: Bound> { pub field_1: T, pub field_2: String, }",
        )
        .unwrap();
        let output = handle_scrypto(attr, item).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(Encode)]
                #[derive(Decode)]
                #[derive(Categorize)]
                #[derive(Describe)]
                #[derive(NonFungibleData)]
                #[sbor(custom_value_kind = "radix_engine_interface::data::ScryptoCustomValueKind")]
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
            TokenStream::from_str("Encode, Decode, Categorize, Describe, NonFungibleData").unwrap();
        let item = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_scrypto(attr, item).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(Encode)]
                #[derive(Decode)]
                #[derive(Categorize)]
                #[derive(Describe)]
                #[derive(NonFungibleData)]
                #[sbor(custom_value_kind = "radix_engine_interface::data::ScryptoCustomValueKind")]
                enum MyEnum<T: Bound> {
                    A { named: T },
                    B(String),
                    C
                }
            },
        );
    }
}
