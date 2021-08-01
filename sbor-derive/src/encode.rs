use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_encode(input: TokenStream) -> TokenStream {
    trace!("handle_encode() starts");

    let DeriveInput { ident, data, .. } = parse2(input).expect("Unable to parse input");
    let ident_str = ident.to_string();
    trace!("Encoding: {}", ident);

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

                quote! {
                    impl sbor::Encode for #ident {
                        fn encode(&self, encoder: &mut sbor::Encoder) {
                            use sbor::{self, Encode};

                            encoder.encode_type(sbor::TYPE_STRUCT);
                            encoder.encode_name(#ident_str);

                            encoder.encode_type(sbor::TYPE_FIELDS_NAMED);
                            encoder.encode_len(#n);
                            #(
                                encoder.encode_name(#names);
                                self.#idents.encode(encoder);
                            )*
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let n = unnamed.len();
                let ith = (0..n).map(|i| Index::from(i));

                quote! {
                    impl sbor::Encode for #ident {
                        fn encode(&self, encoder: &mut sbor::Encoder) {
                            use sbor::{self, Encode};

                            encoder.encode_type(sbor::TYPE_STRUCT);
                            encoder.encode_name(#ident_str);

                            encoder.encode_type(sbor::TYPE_FIELDS_UNNAMED);
                            encoder.encode_len(#n);
                            #(self.#ith.encode(encoder);)*
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl sbor::Encode for #ident {
                        fn encode(&self, encoder: &mut sbor::Encoder) {
                            encoder.encode_type(sbor::TYPE_STRUCT);
                            encoder.encode_name(#ident_str);
                            encoder.encode_type(sbor::TYPE_FIELDS_UNIT);
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
                        let idents2 = named.iter().map(|f| &f.ident);
                        let n = named.len();
                        quote! {
                            Self::#v_id {#(#idents),*} => {
                                encoder.encode_str(#v_name);
                                encoder.encode_type(sbor::TYPE_FIELDS_NAMED);
                                encoder.encode_len(#n);
                                #(
                                    encoder.encode_name(#names);
                                    #idents2.encode(encoder);
                                )*
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let n = unnamed.len() as usize;
                        let args = (0..n).map(|i| format_ident!("a{}", i));
                        let args2 = (0..n).map(|i| format_ident!("a{}", i));
                        quote! {
                            Self::#v_id (#(#args),*) => {
                                encoder.encode_str(#v_name);
                                encoder.encode_type(sbor::TYPE_FIELDS_UNNAMED);
                                encoder.encode_len(#n);
                                #(#args2.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            Self::#v_id => {
                                encoder.encode_str(#v_name);
                                encoder.encode_type(sbor::TYPE_FIELDS_UNIT);
                            }
                        }
                    }
                }
            });

            quote! {
                impl sbor::Encode for #ident {
                    fn encode(&self, encoder: &mut sbor::Encoder) {
                        use sbor::{self, Encode};

                        encoder.encode_type(sbor::TYPE_ENUM);
                        encoder.encode_name(#ident_str);

                        match self {
                            #(#match_arms),*
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
    crate::utils::print_compiled_code("Encode", &output);

    output.into()
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::str::FromStr;
    use proc_macro2::TokenStream;

    use super::handle_encode;

    #[test]
    fn test_encode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        handle_encode(input);
    }

    #[test]
    fn test_encode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        handle_encode(input);
    }
}
