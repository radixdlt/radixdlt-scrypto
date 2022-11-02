use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_scrypto_data(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_scrypto_data() starts");

    let DeriveInput {
        ident,
        data,
        attrs: _,
        vis,
        generics,
    } = parse2(input).expect("Unable to parse input");

    let output = match &data {
        Data::Struct(DataStruct {
            struct_token,
            fields,
            semi_token,
        }) => quote! {
            #[derive(::sbor::Encode, ::sbor::Decode, ::sbor::TypeId, ::scrypto::Describe)]
            #[custom_type_id(::scrypto::data::ScryptoCustomTypeId)]
            #vis #struct_token #ident #generics #fields #semi_token
        },
        Data::Enum(DataEnum {
            enum_token,
            brace_token: _,
            variants,
        }) => quote! {
            #[derive(::sbor::Encode, ::sbor::Decode, ::sbor::TypeId, ::scrypto::Describe)]
            #[custom_type_id(::scrypto::data::ScryptoCustomTypeId)]
            #vis #enum_token #ident #generics { #variants }
        },
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("ScryptoData", &output);

    trace!("handle_scrypto_data() finishes");
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
        let input = TokenStream::from_str(
            "pub struct MyStruct<T: Bound> { pub field_1: T, pub field_2: String, }",
        )
        .unwrap();
        let output = handle_scrypto_data(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::Encode, ::sbor::Decode, ::sbor::TypeId)]
                #[custom_type_id(::scrypto::data::ScryptoCustomTypeId)]
                pub struct MyStruct<T: Bound> {
                    pub field_1: T,
                    pub field_2: String,
                }
            },
        );
    }

    #[test]
    fn test_scrypto_data_with_enum() {
        let input = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_scrypto_data(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::Encode, ::sbor::Decode, ::sbor::TypeId, ::scrypto::Describe)]
                #[custom_type_id(::scrypto::data::ScryptoCustomTypeId)]
                enum MyEnum<T: Bound> {
                    A { named: T },
                    B(String),
                    C,
                }
            },
        );
    }
}
