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
                            extern crate alloc;
                            use alloc::string::ToString;
                            use sbor::{self, Decode};

                            decoder.decode_type_and_check(sbor::TYPE_STRUCT)?;
                            let name = decoder.decode_string()?;
                            let expected = #ident_str.to_string();
                            if name != expected {
                                return Err(format!(
                                    "Unexpected struct name: expected = {}, actual = {}",
                                    expected,
                                    name
                                ));
                            }

                            decoder.decode_type_and_check(sbor::TYPE_FIELDS_NAMED)?;
                            let n = decoder.decode_len()?;
                            if n != #n {
                                return Err(format!(
                                    "Unexpected number of fields: expected = {}, actual = {}",
                                    #n,
                                    n
                                ));
                            }

                            Ok(Self {
                                #(#idents: {
                                    let name = decoder.decode_string()?;
                                    let expected = #names.to_string();
                                    if name != expected {
                                        return Err(format!(
                                            "Unexpected struct field: expected = {}, actual = {}",
                                            expected,
                                            name
                                        ));
                                    }
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
                            extern crate alloc;
                            use alloc::string::ToString;
                            use alloc::vec::Vec;
                            use sbor::{self, Decode};

                            decoder.decode_type_and_check(sbor::TYPE_STRUCT)?;
                            let name = decoder.decode_string()?;
                            let expected = #ident_str.to_string();
                            if name != expected {
                                return Err(format!(
                                    "Unexpected struct name: expected = {}, actual = {}",
                                    expected,
                                    name
                                ));
                            }

                            decoder.decode_type_and_check(sbor::TYPE_FIELDS_UNNAMED)?;
                            let n = decoder.decode_len()?;
                            if n != #n {
                                return Err(format!(
                                    "Unexpected number of fields: expected = {}, actual = {}",
                                    #n,
                                    n
                                ));
                            }

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
                            extern crate alloc;
                            use alloc::string::ToString;

                            decoder.decode_type_and_check(sbor::TYPE_STRUCT)?;
                            let name = decoder.decode_string()?;
                            let expected = #ident_str.to_string();
                            if name != expected {
                                return Err(format!(
                                    "Unexpected struct name: expected = {}, actual = {}",
                                    expected,
                                    name
                                ));
                            }

                            decoder.decode_type_and_check(sbor::TYPE_FIELDS_UNIT)?;

                            Ok(Self {})
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().map(|v| {
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
                            #v_name => {
                                decoder.decode_type_and_check(sbor::TYPE_FIELDS_NAMED)?;
                                let n = decoder.decode_len()?;
                                if n != #n {
                                    return Err(format!(
                                        "Unexpected number of fields: expected = {}, actual = {}",
                                        #n,
                                        n
                                    ));
                                }

                                Ok(Self::#v_id {
                                    #(#idents: {
                                        let name = decoder.decode_string()?;
                                        let expected = #names.to_string();
                                        if name != expected {
                                            return Err(format!(
                                                "Unexpected struct field: expected = {}, actual = {}",
                                                expected,
                                                name
                                            ));
                                        }
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
                            #v_name => {
                                decoder.decode_type_and_check(sbor::TYPE_FIELDS_UNNAMED)?;
                                let n = decoder.decode_len()?;
                                if n != #n {
                                    return Err(format!(
                                        "Unexpected number of fields: expected = {}, actual = {}",
                                        #n,
                                        n
                                    ));
                                }

                                Ok(Self::#v_id (
                                    #(<#types>::decode(decoder)?),*
                                ))
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            #v_name => {
                                decoder.decode_type_and_check(sbor::TYPE_FIELDS_UNIT)?;
                                Ok(Self::#v_id)
                            }
                        }
                    }
                }
            });

            quote! {
                impl sbor::Decode for #ident {
                    fn decode<'de>(decoder: &'de mut sbor::Decoder) -> Result<Self, String> {
                        extern crate alloc;
                        use alloc::string::ToString;
                        use sbor::{self, Decode};

                        decoder.decode_type_and_check(sbor::TYPE_ENUM)?;
                        let name = decoder.decode_string()?;
                        let expected = #ident_str.to_string();
                        if name != expected {
                            return Err(format!(
                                "Unexpected enum name: expected = {}, actual = {}",
                                expected,
                                name
                            ));
                        }

                        let variant_name = decoder.decode_string()?;
                        match variant_name.as_str() {
                            #(#match_arms,)*
                            _ => Err(format!("Unknown variant: {}", variant_name)),
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
