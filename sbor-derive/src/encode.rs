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
                            extern crate alloc;
                            use alloc::string::ToString;
                            use sbor::{self, Encode};

                            encoder.encode_type(sbor::TYPE_STRUCT);
                            encoder.encode_string(&#ident_str.to_string());

                            encoder.encode_type(sbor::TYPE_FIELDS_NAMED);
                            encoder.encode_len(#n);
                            #(#names.to_string().encode(encoder);)*
                            #(self.#idents.encode(encoder);)*
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
                            extern crate alloc;
                            use alloc::string::ToString;
                            use alloc::vec::Vec;
                            use sbor::{self, Encode};

                            encoder.encode_type(sbor::TYPE_STRUCT);
                            encoder.encode_string(&#ident_str.to_string());

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
                            extern crate alloc;
                            use alloc::string::ToString;

                            encoder.encode_type(sbor::TYPE_STRUCT);
                            encoder.encode_string(&#ident_str.to_string());
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
                        let idents: Vec<_> =
                            named.iter().map(|f| f.ident.clone().unwrap()).collect();
                        let idents2 = idents.clone();
                        let n = named.len();
                        quote! {
                            Self::#v_id {#(#idents),*} => {
                                encoder.encode_string(&#v_name.to_string());
                                encoder.encode_type(sbor::TYPE_FIELDS_NAMED);
                                encoder.encode_len(#n);
                                #(encoder.encode_string(&#names.to_string());)*
                                #(#idents2.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let n = unnamed.len() as usize;
                        let args: Vec<_> = (0..n).map(|i| format_ident!("a{}", i)).collect();
                        let args2 = args.clone();
                        quote! {
                            Self::#v_id (#(#args),*) => {
                                encoder.encode_string(&#v_name.to_string());
                                encoder.encode_type(sbor::TYPE_FIELDS_UNNAMED);
                                encoder.encode_len(#n);
                                #(#args2.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            Self::#v_id => {
                                encoder.encode_string(&#v_name.to_string());
                                encoder.encode_type(sbor::TYPE_FIELDS_UNIT);
                            }
                        }
                    }
                }
            });

            quote! {
                impl sbor::Encode for #ident {
                    fn encode(&self, encoder: &mut sbor::Encoder) {
                        extern crate alloc;
                        use alloc::string::ToString;
                        use sbor::{self, Encode};

                        encoder.encode_type(sbor::TYPE_ENUM);
                        encoder.encode_string(&#ident_str.to_string());

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

    output.into()
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::str::FromStr;
    use proc_macro2::TokenStream;

    use crate::encode::handle_encode;
    use crate::utils::print_compiled_code;

    #[test]
    fn test_encode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_encode(input);
        print_compiled_code("test_encode()", output);
    }

    #[test]
    fn test_encode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_encode(input);
        print_compiled_code("test_encode_enum()", output);
    }
}
