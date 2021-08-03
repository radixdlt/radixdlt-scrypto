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
                        fn encode_value(&self, encoder: &mut sbor::Encoder) {
                            use sbor::{self, Encode};

                            encoder.write_name(#ident_str);
                            encoder.write_type(sbor::TYPE_FIELDS_NAMED);
                            encoder.write_len(#n);
                            #(
                                encoder.write_name(#names);
                                self.#idents.encode(encoder);
                            )*
                        }

                        fn sbor_type() -> u8 {
                            sbor::TYPE_STRUCT
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let n = unnamed.len();
                let ith = (0..n).map(|i| Index::from(i));

                quote! {
                    impl sbor::Encode for #ident {
                        fn encode_value(&self, encoder: &mut sbor::Encoder) {
                            use sbor::{self, Encode};

                            encoder.write_name(#ident_str);
                            encoder.write_type(sbor::TYPE_FIELDS_UNNAMED);
                            encoder.write_len(#n);
                            #(self.#ith.encode(encoder);)*
                        }

                        fn sbor_type() -> u8 {
                            sbor::TYPE_STRUCT
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl sbor::Encode for #ident {
                        fn encode_value(&self, encoder: &mut sbor::Encoder) {
                            encoder.write_name(#ident_str);
                            encoder.write_type(sbor::TYPE_FIELDS_UNIT);
                        }

                        fn sbor_type() -> u8 {
                            sbor::TYPE_STRUCT
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
                        let idents2 = named.iter().map(|f| &f.ident);
                        let n = named.len();
                        quote! {
                            Self::#v_id {#(#idents),*} => {
                                encoder.write_index(#v_ith);
                                encoder.write_name(#v_name);
                                encoder.write_type(sbor::TYPE_FIELDS_NAMED);
                                encoder.write_len(#n);
                                #(
                                    encoder.write_name(#names);
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
                                encoder.write_index(#v_ith);
                                encoder.write_name(#v_name);
                                encoder.write_type(sbor::TYPE_FIELDS_UNNAMED);
                                encoder.write_len(#n);
                                #(#args2.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            Self::#v_id => {
                                encoder.write_index(#v_ith);
                                encoder.write_name(#v_name);
                                encoder.write_type(sbor::TYPE_FIELDS_UNIT);
                            }
                        }
                    }
                }
            });

            quote! {
                impl sbor::Encode for #ident {
                    fn encode_value(&self, encoder: &mut sbor::Encoder) {
                        use sbor::{self, Encode};

                        encoder.write_name(#ident_str);
                        match self {
                            #(#match_arms)*
                        }
                    }

                    fn sbor_type() -> u8 {
                        sbor::TYPE_ENUM
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

    use super::*;
    use proc_macro2::TokenStream;

    fn code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_encode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_encode(input);

        code_eq(
            output,
            quote! {
                impl sbor::Encode for Test {
                    fn encode_value(&self, encoder: &mut sbor::Encoder) {
                        use sbor::{self, Encode};
                        encoder.write_name("Test");
                        encoder.write_type(sbor::TYPE_FIELDS_NAMED);
                        encoder.write_len(1usize);
                        encoder.write_name("a");
                        self.a.encode(encoder);
                    }
                    fn sbor_type() -> u8 {
                        sbor::TYPE_STRUCT
                    }
                }
            },
        );
    }

    #[test]
    fn test_encode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_encode(input);

        code_eq(
            output,
            quote! {
                impl sbor::Encode for Test {
                    fn encode_value(&self, encoder: &mut sbor::Encoder) {
                        use sbor::{self, Encode};
                        encoder.write_name("Test");
                        match self {
                            Self::A => {
                                encoder.write_index(0usize);
                                encoder.write_name("A");
                                encoder.write_type(sbor::TYPE_FIELDS_UNIT);
                            }
                            Self::B(a0) => {
                                encoder.write_index(1usize);
                                encoder.write_name("B");
                                encoder.write_type(sbor::TYPE_FIELDS_UNNAMED);
                                encoder.write_len(1usize);
                                a0.encode(encoder);
                            }
                            Self::C { x } => {
                                encoder.write_index(2usize);
                                encoder.write_name("C");
                                encoder.write_type(sbor::TYPE_FIELDS_NAMED);
                                encoder.write_len(1usize);
                                encoder.write_name("x");
                                x.encode(encoder);
                            }
                        }
                    }
                    fn sbor_type() -> u8 {
                        sbor::TYPE_ENUM
                    }
                }
            },
        );
    }
}
