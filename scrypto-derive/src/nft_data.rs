use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

fn is_mutable(f: &syn::Field) -> bool {
    let mut mutable = false;
    for att in &f.attrs {
        if att.path.is_ident("scrypto")
            && att
                .parse_args::<syn::Path>()
                .map(|p| p.is_ident("mutable"))
                .unwrap_or(false)
        {
            mutable = true;
        }
    }
    mutable
}

pub fn handle_nft_data(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_nft_data() starts");

    let DeriveInput { ident, data, .. } = parse2(input).expect("Unable to parse input");
    let ident_str = ident.to_string();
    trace!("Processing: {}", ident_str);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // immutable
                let im: Vec<&Field> = named.iter().filter(|f| !is_mutable(f)).collect();
                let im_n = Index::from(im.len());
                let im_ids = im.iter().map(|f| &f.ident);
                let im_ids2 = im_ids.clone();
                let im_types = im.iter().map(|f| &f.ty);
                let im_types2 = im_types.clone();
                let im_names = im
                    .iter()
                    .map(|f| f.ident.clone().expect("Illegal State!").to_string());
                // mutable
                let m: Vec<&Field> = named.iter().filter(|f| is_mutable(f)).collect();
                let m_n = Index::from(m.len());
                let m_ids = m.iter().map(|f| &f.ident);
                let m_ids2 = m_ids.clone();
                let m_types = m.iter().map(|f| &f.ty);
                let m_types2 = m_types.clone();
                let m_names = m
                    .iter()
                    .map(|f| f.ident.clone().expect("Illegal State!").to_string());

                quote! {
                    impl ::scrypto::resource::NftData for #ident {
                        fn decode(immutable_data: &[u8], mutable_data: &[u8]) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{type_id::*, *};
                            let mut decoder_nm = Decoder::new(immutable_data, true);
                            decoder_nm.check_type(TYPE_FIELDS_NAMED)?;
                            decoder_nm.check_len(#im_n)?;

                            let mut decoder_m = Decoder::new(mutable_data, true);
                            decoder_m.check_type(TYPE_FIELDS_NAMED)?;
                            decoder_m.check_len(#m_n)?;

                            let decoded = Self {
                                #(#im_ids: <#im_types>::decode(&mut decoder_nm)?,)*
                                #(#m_ids: <#m_types>::decode(&mut decoder_m)?,)*
                            };

                            decoder_nm.check_end()?;
                            decoder_m.check_end()?;

                            Ok(decoded)
                        }

                        fn immutable_data(&self) -> ::scrypto::rust::vec::Vec<u8> {
                            use ::sbor::{type_id::*, *};

                            let mut encoder = Encoder::new(Vec::new(), true);
                            encoder.write_type(TYPE_FIELDS_NAMED);
                            encoder.write_len(#im_n);
                            #(
                                self.#im_ids2.encode(&mut encoder);
                            )*

                            encoder.into()
                        }

                        fn mutable_data(&self) -> ::scrypto::rust::vec::Vec<u8> {
                            use ::sbor::{type_id::*, *};
                            use ::scrypto::rust::vec::Vec;

                            let mut encoder = Encoder::new(Vec::new(), true);
                            encoder.write_type(TYPE_FIELDS_NAMED);
                            encoder.write_len(#m_n);
                            #(
                                self.#m_ids2.encode(&mut encoder);
                            )*

                            encoder.into()
                        }

                        fn immutable_data_schema(&self) -> ::sbor::describe::Type {
                            use ::sbor::rust::borrow::ToOwned;
                            use ::sbor::rust::vec;
                            use ::sbor::Describe;

                            ::sbor::describe::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::sbor::describe::Fields::Named {
                                    named: vec![#((#im_names.to_owned(), <#im_types2>::describe())),*]
                                },
                            }
                        }

                        fn mutable_data_schema(&self) -> ::sbor::describe::Type {
                            use ::sbor::rust::borrow::ToOwned;
                            use ::sbor::rust::vec;
                            use ::sbor::Describe;

                            ::sbor::describe::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::sbor::describe::Fields::Named {
                                    named: vec![#((#m_names.to_owned(), <#m_types2>::describe())),*]
                                },
                            }
                        }
                    }
                }
            }
            syn::Fields::Unnamed(_) => {
                panic!("Struct with unnamed fields is not supported!")
            }
            syn::Fields::Unit => {
                panic!("Struct with no fields is not supported!")
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            panic!("Union is not supported!")
        }
    };
    trace!("handle_nft_data() finishes");

    //#[cfg(feature = "trace")]
    crate::utils::print_compiled_code("NftData", &output);

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
    fn test_nft() {
        let input = TokenStream::from_str(
            "pub struct AwesomeNftData { pub field_1: u32, #[scrypto(mutable)] pub field_2: String, }",
        )
        .unwrap();
        let output = handle_nft_data(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::scrypto::resource::NftData for AwesomeNftData {
                    fn decode(immutable_data: &[u8], mutable_data: &[u8]) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{type_id::*, *};
                        let mut decoder_nm = Decoder::new(immutable_data, true);
                        decoder_nm.check_type(TYPE_FIELDS_NAMED)?;
                        decoder_nm.check_len(1)?;
                        let mut decoder_m = Decoder::new(mutable_data, true);
                        decoder_m.check_type(TYPE_FIELDS_NAMED)?;
                        decoder_m.check_len(1)?;
                        let decoded = Self {
                            field_1: <u32>::decode(&mut decoder_nm)?,
                            field_2: <String>::decode(&mut decoder_m)?,
                        };
                        decoder_nm.check_end()?;
                        decoder_m.check_end()?;
                        Ok(decoded)
                    }
                    fn immutable_data(&self) -> ::scrypto::rust::vec::Vec<u8> {
                        use ::sbor::{type_id::*, *};
                        let mut encoder = Encoder::new(Vec::new(), true);
                        encoder.write_type(TYPE_FIELDS_NAMED);
                        encoder.write_len(1);
                        self.field_1.encode(&mut encoder);
                        encoder.into()
                    }
                    fn mutable_data(&self) -> ::scrypto::rust::vec::Vec<u8> {
                        use ::sbor::{type_id::*, *};
                        use ::scrypto::rust::vec::Vec;
                        let mut encoder = Encoder::new(Vec::new(), true);
                        encoder.write_type(TYPE_FIELDS_NAMED);
                        encoder.write_len(1);
                        self.field_2.encode(&mut encoder);
                        encoder.into()
                    }
                    fn immutable_data_schema(&self) -> ::sbor::describe::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::sbor::Describe;
                        ::sbor::describe::Type::Struct {
                            name: "AwesomeNftData".to_owned(),
                            fields: ::sbor::describe::Fields::Named {
                                named: vec![("field_1".to_owned(), <u32>::describe())]
                            },
                        }
                    }
                    fn mutable_data_schema(&self) -> ::sbor::describe::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::sbor::Describe;
                        ::sbor::describe::Type::Struct {
                            name: "AwesomeNftData".to_owned(),
                            fields: ::sbor::describe::Fields::Named {
                                named: vec![("field_2".to_owned(), <String>::describe())]
                            },
                        }
                    }
                }
            },
        );
    }
}
