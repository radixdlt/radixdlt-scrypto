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
    let ident_str = ident.to_string();
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
                    impl sbor::Decode for #ident {
                        fn decode<'de>(decoder: &'de mut sbor::Decoder) -> Result<Self, String> {
                            use sbor::{self, Decode};

                            decoder.check_type(sbor::TYPE_STRUCT)?;
                            decoder.check_name(#ident_str)?;

                            decoder.check_type(sbor::TYPE_FIELDS_NAMED)?;
                            decoder.check_len(#n)?;

                            Ok(Self {
                                #(#idents: {
                                    decoder.check_name(#names)?;
                                    <#types>::decode(decoder)?
                                }),*
                            })
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let n = unnamed.len();
                let types = unnamed.iter().map(|f| &f.ty);

                quote! {
                    impl sbor::Decode for #ident {
                        fn decode<'de>(decoder: &'de mut sbor::Decoder) -> Result<Self, String> {
                            use sbor::{self, Decode};

                            decoder.check_type(sbor::TYPE_STRUCT)?;
                            decoder.check_name(#ident_str)?;

                            decoder.check_type(sbor::TYPE_FIELDS_UNNAMED)?;
                            decoder.check_len(#n)?;

                            Ok(Self (
                                #(<#types>::decode(decoder)?),*
                            ))
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl sbor::Decode for #ident {
                        fn decode<'de>(decoder: &'de mut sbor::Decoder) -> Result<Self, String> {
                            decoder.check_type(sbor::TYPE_STRUCT)?;
                            decoder.check_name(#ident_str)?;

                            decoder.check_type(sbor::TYPE_FIELDS_UNIT)?;

                            Ok(Self {})
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().enumerate().map(|(v_ith, v)| {
                let v_id = &v.ident;
                let v_name = v_id.to_string();
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
                                decoder.check_type(sbor::TYPE_FIELDS_NAMED)?;
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
                                decoder.check_type(sbor::TYPE_FIELDS_UNNAMED)?;
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
                                decoder.check_type(sbor::TYPE_FIELDS_UNIT)?;
                                Ok(Self::#v_id)
                            }
                        }
                    }
                }
            });

            quote! {
                impl sbor::Decode for #ident {
                    fn decode<'de>(decoder: &'de mut sbor::Decoder) -> Result<Self, String> {
                        use sbor::{self, Decode};

                        decoder.check_type(sbor::TYPE_ENUM)?;
                        decoder.check_name(#ident_str)?;

                        let index = decoder.read_index()?;
                        match index {
                            #(#match_arms,)*
                            _ => Err(format!("Unknown enum index: {}", index)),
                        }
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
    extern crate alloc;
    use alloc::str::FromStr;
    use proc_macro2::TokenStream;

    use super::handle_decode;

    #[test]
    fn test_decode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        handle_decode(input);
    }

    #[test]
    fn test_decode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        handle_decode(input);
    }
}
