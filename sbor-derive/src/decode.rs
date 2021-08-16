use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_decode(input: TokenStream) -> TokenStream {
    trace!("handle_decode() starts");

    let DeriveInput { ident, data, .. } = parse2(input).expect("Unable to parse input");
    trace!("Decoding: {}", ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let n = named.len();
                let names = named.iter().map(|f| {
                    f.ident
                        .clone()
                        .expect("All fields must be named")
                        .to_string()
                });
                let idents = named.iter().map(|f| &f.ident);
                let types = named.iter().map(|f| &f.ty);

                quote! {
                    impl ::sbor::Decode for #ident {
                        fn decode_value<'de>(decoder: &'de mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{self, Decode};

                            decoder.check_type(::sbor::constants::TYPE_FIELDS_NAMED)?;
                            decoder.check_len(#n)?;

                            Ok(Self {
                                #(#idents: {
                                    decoder.check_name(#names)?;
                                    <#types>::decode(decoder)?
                                }),*
                            })
                        }

                        fn sbor_type() -> u8 {
                            ::sbor::constants::TYPE_STRUCT
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let n = unnamed.len();
                let types = unnamed.iter().map(|f| &f.ty);

                quote! {
                    impl ::sbor::Decode for #ident {
                        fn decode_value<'de>(decoder: &'de mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{self, Decode};

                            decoder.check_type(::sbor::constants::TYPE_FIELDS_UNNAMED)?;
                            decoder.check_len(#n)?;

                            Ok(Self (
                                #(<#types>::decode(decoder)?),*
                            ))
                        }

                        fn sbor_type() -> u8 {
                            ::sbor::constants::TYPE_STRUCT
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl ::sbor::Decode for #ident {
                        fn decode_value<'de>(decoder: &'de mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                            decoder.check_type(::sbor::constants::TYPE_FIELDS_UNIT)?;

                            Ok(Self {})
                        }

                        fn sbor_type() -> u8 {
                            ::sbor::constants::TYPE_STRUCT
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().enumerate().map(|(i, v)| {
                let v_id = &v.ident;
                let v_name = v_id.to_string();
                let v_ith = i as u8;
                match &v.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let names = named.iter().map(|f| {
                            f.ident
                                .clone()
                                .expect("All fields must be named")
                                .to_string()
                        });
                        let idents = named.iter().map(|f| &f.ident);
                        let types = named.iter().map(|f| &f.ty);
                        let n = named.len();
                        quote! {
                            #v_ith => {
                                decoder.check_name(#v_name)?;
                                decoder.check_type(::sbor::constants::TYPE_FIELDS_NAMED)?;
                                decoder.check_len(#n)?;

                                Ok(Self::#v_id {
                                    #(#idents: {
                                        decoder.check_name(#names)?;
                                        <#types>::decode(decoder)?
                                    }),*
                                })
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let n = unnamed.len() as usize;
                        let types = unnamed.iter().map(|f| &f.ty);
                        quote! {
                            #v_ith => {
                                decoder.check_name(#v_name)?;
                                decoder.check_type(::sbor::constants::TYPE_FIELDS_UNNAMED)?;
                                decoder.check_len(#n)?;

                                Ok(Self::#v_id (
                                    #(<#types>::decode(decoder)?),*
                                ))
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            #v_ith => {
                                decoder.check_name(#v_name)?;
                                decoder.check_type(::sbor::constants::TYPE_FIELDS_UNIT)?;
                                Ok(Self::#v_id)
                            }
                        }
                    }
                }
            });

            quote! {
                impl ::sbor::Decode for #ident {
                    #[inline]
                    fn decode_value<'de>(decoder: &'de mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};

                        let index = decoder.read_index()?;
                        match index {
                            #(#match_arms,)*
                            _ => Err(::sbor::DecodeError::InvalidIndex(index))
                        }
                    }

                    #[inline]
                    fn sbor_type() -> u8 {
                        ::sbor::constants::TYPE_ENUM
                    }
                }
            }
        }
        Data::Union(_) => {
            panic!("Union is not supported!")
        }
    };
    trace!("handle_derive() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("Decode", &output);

    output.into()
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
    fn test_decode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_decode(input);

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Decode for Test {
                    fn decode_value<'de>(decoder: &'de mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_type(::sbor::constants::TYPE_FIELDS_NAMED)?;
                        decoder.check_len(1usize)?;
                        Ok(Self {
                            a: {
                                decoder.check_name("a")?;
                                <u32>::decode(decoder)?
                            }
                        })
                    }
                    fn sbor_type() -> u8 {
                        ::sbor::constants::TYPE_STRUCT
                    }
                }
            },
        );
    }

    #[test]
    fn test_decode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_decode(input);

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Decode for Test {
                    #[inline]
                    fn decode_value<'de>(decoder: &'de mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        let index = decoder.read_index()?;
                        match index {
                            0u8 => {
                                decoder.check_name("A")?;
                                decoder.check_type(::sbor::constants::TYPE_FIELDS_UNIT)?;
                                Ok(Self::A)
                            },
                            1u8 => {
                                decoder.check_name("B")?;
                                decoder.check_type(::sbor::constants::TYPE_FIELDS_UNNAMED)?;
                                decoder.check_len(1usize)?;
                                Ok(Self::B(<u32>::decode(decoder)?))
                            },
                            2u8 => {
                                decoder.check_name("C")?;
                                decoder.check_type(::sbor::constants::TYPE_FIELDS_NAMED)?;
                                decoder.check_len(1usize)?;
                                Ok(Self::C {
                                    x: {
                                        decoder.check_name("x")?;
                                        <u8>::decode(decoder)?
                                    }
                                })
                            },
                            _ => Err(::sbor::DecodeError::InvalidIndex(index))
                        }
                    }
                    #[inline]
                    fn sbor_type() -> u8 {
                        ::sbor::constants::TYPE_ENUM
                    }
                }
            },
        );
    }
}
