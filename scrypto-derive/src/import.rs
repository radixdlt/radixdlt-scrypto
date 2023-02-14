use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::*;

use scrypto_abi as abi;
use scrypto_abi::Fields as SchemaFields;
use scrypto_abi::Type as SchemaType;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_import(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_import() starts");

    let content = parse2::<LitStr>(input)?;
    let blueprint: abi::Blueprint = match serde_json::from_str(content.value().as_str()) {
        Ok(o) => o,
        Err(e) => {
            return Err(Error::new(content.span(), e));
        }
    };
    trace!("Parsed ABI: {:?}", blueprint);

    let package_address = blueprint.package_address;
    let blueprint_name = blueprint.blueprint_name;
    let ident = format_ident!("{}GlobalComponentRef", blueprint_name);
    trace!("Blueprint name: {}", blueprint_name);

    let mut structs: Vec<Item> = vec![];

    let mut fns = Vec::<ItemFn>::new();
    for function in &blueprint.abi.fns {
        trace!("Processing function: {:?}", function);

        let func_name = &function.ident;
        let func_indent = format_ident!("{}", func_name);
        let mut func_types = Vec::<Type>::new();
        let mut func_args = Vec::<Ident>::new();

        match &function.input {
            SchemaType::Struct {
                name: _,
                fields: SchemaFields::Named { named },
            } => {
                for (i, (_, input)) in named.iter().enumerate() {
                    let ident = format_ident!("arg{}", i);
                    let (new_type, new_structs) = get_native_type(input)?;
                    func_args.push(parse_quote! { #ident });
                    func_types.push(parse_quote! { #new_type });
                    structs.extend(new_structs);
                }
            }
            _ => panic!("Cannot construct abi"),
        }

        let (func_output, new_structs) = get_native_type(&function.output)?;
        structs.extend(new_structs);

        if let None = function.mutability {
            fns.push(parse_quote! {
                pub fn #func_indent(#(#func_args: #func_types),*) -> #func_output {
                    ::scrypto::runtime::Runtime::call_function(
                        ::scrypto::model::PackageAddress::try_from_hex(#package_address).unwrap(),
                        #blueprint_name,
                        #func_name,
                        args!(#(#func_args),*)
                    )
                }
            });
        } else {
            fns.push(parse_quote! {
                pub fn #func_indent(&self #(, #func_args: #func_types)*) -> #func_output {
                    ::scrypto::runtime::Runtime::call_method(
                        self.component_address,
                        #func_name,
                        args!(#(#func_args),*)
                    )
                }
            });
        }
    }

    let output = quote! {
        #(#structs)*

        #[derive(::sbor::Categorize, ::sbor::Encode, ::sbor::Decode, ::scrypto::LegacyDescribe)]
        #[sbor(custom_value_kind = "::scrypto::data::ScryptoCustomValueKind")]
        pub struct #ident {
            component_address: ::scrypto::model::ComponentAddress,
        }

        impl #ident {
            #(#fns)*
        }

        impl From<::scrypto::model::ComponentAddress> for #ident {
            fn from(component_address: ::scrypto::model::ComponentAddress) -> Self {
                Self {
                    component_address
                }
            }
        }

        impl From<#ident> for ::scrypto::model::ComponentAddress {
            fn from(a: #ident) -> ::scrypto::model::ComponentAddress {
                a.component_address
            }
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("import!", &output);

    trace!("handle_import() finishes");
    Ok(output)
}

fn get_native_type(ty: &SchemaType) -> Result<(Type, Vec<Item>)> {
    let mut structs = Vec::<Item>::new();

    let t: Type = match ty {
        // primitive types
        SchemaType::Bool => parse_quote! { bool },
        SchemaType::I8 => parse_quote! { i8 },
        SchemaType::I16 => parse_quote! { i16 },
        SchemaType::I32 => parse_quote! { i32 },
        SchemaType::I64 => parse_quote! { i64 },
        SchemaType::I128 => parse_quote! { i128 },
        SchemaType::U8 => parse_quote! { u8 },
        SchemaType::U16 => parse_quote! { u16 },
        SchemaType::U32 => parse_quote! { u32 },
        SchemaType::U64 => parse_quote! { u64 },
        SchemaType::U128 => parse_quote! { u128 },
        SchemaType::String => parse_quote! { String },

        // array
        SchemaType::Array {
            element_type,
            length,
        } => {
            let (new_type, new_structs) = get_native_type(element_type)?;
            structs.extend(new_structs);

            let n = *length as usize;
            parse_quote! { [#new_type; #n] }
        }
        SchemaType::Vec { element_type } => {
            let (new_type, new_structs) = get_native_type(element_type)?;
            structs.extend(new_structs);

            parse_quote! { Vec<#new_type> }
        }
        SchemaType::TreeSet { element_type } => {
            let (new_type, new_structs) = get_native_type(element_type)?;
            structs.extend(new_structs);

            parse_quote! { BTreeSet<#new_type> }
        }
        SchemaType::TreeMap {
            key_type,
            value_type,
        } => {
            let (key_type, new_structs) = get_native_type(key_type)?;
            structs.extend(new_structs);
            let (value_type, new_structs) = get_native_type(value_type)?;
            structs.extend(new_structs);

            parse_quote! { BTreeMap<#key_type, #value_type> }
        }
        SchemaType::HashSet { element_type } => {
            let (new_type, new_structs) = get_native_type(element_type)?;
            structs.extend(new_structs);

            parse_quote! { HashSet<#new_type> }
        }
        SchemaType::HashMap {
            key_type,
            value_type,
        } => {
            let (key_type, new_structs) = get_native_type(key_type)?;
            structs.extend(new_structs);
            let (value_type, new_structs) = get_native_type(value_type)?;
            structs.extend(new_structs);

            parse_quote! { HashMap<#key_type, #value_type> }
        }

        // tuple
        SchemaType::Tuple { element_types } => {
            let mut types: Vec<Type> = vec![];

            for element_type in element_types {
                let (new_type, new_structs) = get_native_type(element_type)?;
                types.push(new_type);
                structs.extend(new_structs);
            }

            parse_quote! { ( #(#types),* ) }
        }
        SchemaType::NonFungibleGlobalId => {
            parse_quote! { ::scrypto::model::NonFungibleGlobalId}
        }
        SchemaType::Struct { name, fields } => {
            let ident = format_ident!("{}", name);

            match fields {
                SchemaFields::Named { named } => {
                    let names: Vec<Ident> =
                        named.iter().map(|k| format_ident!("{}", k.0)).collect();
                    let mut types: Vec<Type> = vec![];
                    for (_, v) in named {
                        let (new_type, new_structs) = get_native_type(v)?;
                        types.push(new_type);
                        structs.extend(new_structs);
                    }
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::Categorize, ::sbor::Encode, ::sbor::Decode, ::scrypto::LegacyDescribe)]
                        #[sbor(custom_value_kind = "::scrypto::data::ScryptoCustomValueKind")]
                        pub struct #ident {
                            #( pub #names : #types, )*
                        }
                    });
                }
                SchemaFields::Unnamed { unnamed } => {
                    let mut types: Vec<Type> = vec![];
                    for v in unnamed {
                        let (new_type, new_structs) = get_native_type(v)?;
                        types.push(new_type);
                        structs.extend(new_structs);
                    }
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::Categorize, ::sbor::Encode, ::sbor::Decode, ::scrypto::LegacyDescribe)]
                        #[sbor(custom_value_kind = "::scrypto::data::ScryptoCustomValueKind")]
                        pub struct #ident (
                            #( pub #types ),*
                        );
                    });
                }
                SchemaFields::Unit => {
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::Categorize, ::sbor::Encode, ::sbor::Decode, ::scrypto::LegacyDescribe)]
                        #[sbor(custom_value_kind = "::scrypto::data::ScryptoCustomValueKind")]
                        pub struct #ident;
                    });
                }
            }

            parse_quote! { #ident }
        }

        // enums
        SchemaType::Enum { name, variants } => {
            let ident = format_ident!("{}", name);
            let mut native_variants = Vec::<Variant>::new();

            for variant in variants {
                let v_ident = format_ident!("{}", variant.name);

                match &variant.fields {
                    SchemaFields::Named { named } => {
                        let mut names: Vec<Ident> = vec![];
                        let mut types: Vec<Type> = vec![];
                        for (n, v) in named {
                            names.push(format_ident!("{}", n));
                            let (new_type, new_structs) = get_native_type(v)?;
                            types.push(new_type);
                            structs.extend(new_structs);
                        }
                        native_variants.push(parse_quote! {
                            #v_ident {
                                #(#names: #types),*
                            }
                        });
                    }
                    SchemaFields::Unnamed { unnamed } => {
                        let mut types: Vec<Type> = vec![];
                        for v in unnamed {
                            let (new_type, new_structs) = get_native_type(v)?;
                            types.push(new_type);
                            structs.extend(new_structs);
                        }
                        native_variants.push(parse_quote! {
                            #v_ident ( #(#types),* )
                        });
                    }
                    SchemaFields::Unit => {
                        native_variants.push(parse_quote! {
                            #v_ident
                        });
                    }
                };
            }

            structs.push(parse_quote! {
                #[derive(Debug, ::sbor::Categorize, ::sbor::Encode, ::sbor::Decode, ::scrypto::LegacyDescribe)]
                #[sbor(custom_value_kind = "::scrypto::data::ScryptoCustomValueKind")]
                pub enum #ident {
                    #( #native_variants ),*
                }
            });

            parse_quote! { #ident }
        }
        SchemaType::Option { some_type } => {
            let (new_type, new_structs) = get_native_type(some_type)?;
            structs.extend(new_structs);

            parse_quote! { Option<#new_type> }
        }
        SchemaType::Result {
            okay_type,
            err_type,
        } => {
            let (okay_type, new_structs) = get_native_type(okay_type)?;
            structs.extend(new_structs);
            let (err_type, new_structs) = get_native_type(err_type)?;
            structs.extend(new_structs);

            parse_quote! { Result<#okay_type, #err_type> }
        }

        // RE
        SchemaType::PackageAddress => {
            parse_quote! { ::scrypto::model::PackageAddress }
        }
        SchemaType::ComponentAddress => {
            parse_quote! { ::scrypto::model::ComponentAddress}
        }
        SchemaType::ResourceAddress => {
            parse_quote! {::scrypto::model::ResourceAddress }
        }
        SchemaType::Own => parse_quote! { ::scrypto::radix_engine_interface::data::types::Own },
        SchemaType::Bucket => parse_quote! {::scrypto::model::Bucket },
        SchemaType::Proof => parse_quote! { ::scrypto::model::Proof},
        SchemaType::Vault => parse_quote! { ::scrypto::model::Vault},
        SchemaType::Component => parse_quote! {::scrypto::model::Component },
        SchemaType::KeyValueStore {
            key_type,
            value_type,
        } => {
            let (k, s) = get_native_type(key_type)?;
            structs.extend(s);
            let (v, s) = get_native_type(value_type)?;
            structs.extend(s);
            parse_quote! { ::scrypto::component::KeyValueStore<#k, #v> }
        }

        // Misc
        SchemaType::Hash => parse_quote! { ::scrypto::crypto::Hash},
        SchemaType::EcdsaSecp256k1PublicKey => {
            parse_quote! {::scrypto::crypto::EcdsaSecp256k1PublicKey }
        }
        SchemaType::EcdsaSecp256k1Signature => {
            parse_quote! { ::scrypto::crypto::EcdsaSecp256k1Signature}
        }
        SchemaType::EddsaEd25519PublicKey => {
            parse_quote! { ::scrypto::crypto::EddsaEd25519PublicKey}
        }
        SchemaType::EddsaEd25519Signature => {
            parse_quote! {::scrypto::crypto::EddsaEd25519Signature }
        }
        SchemaType::Decimal => parse_quote! { ::scrypto::math::Decimal},
        SchemaType::PreciseDecimal => parse_quote! {::scrypto::math::PreciseDecimal },
        SchemaType::NonFungibleLocalId => parse_quote! {::scrypto::model::NonFungibleLocalId },

        SchemaType::Any => {
            panic!("Any type not currently supported for importing.");
        }
    };

    Ok((t, structs))
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
    fn test_import_empty() {
        let input = TokenStream::from_str(
            r###"
                r#"
                {
                    "package_address": "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7",
                    "blueprint_name": "Simple",
                    "abi": {
                        "structure": {
                            "type": "Struct",
                            "name": "Simple",
                            "fields": {
                                "type": "Named",
                                "named": []
                            }
                        },
                        "fns": [
                            {
                                "ident": "new",
                                "input": {
                                    "type": "Struct",
                                    "name": "",
                                    "fields": {
                                        "type": "Named",
                                        "named": []
                                    }
                                },
                                "output": {
                                    "type": "ComponentAddress"
                                },
                                "export_name": "Simple_new_main"
                            },
                            {
                                "ident": "free_token",
                                "mutability": "Mutable",
                                "input": {
                                    "type": "Struct",
                                    "name": "",
                                    "fields": {
                                        "type": "Named",
                                        "named": []
                                    }
                                },
                                "output": {
                                    "type": "Bucket"
                                },
                                "export_name": "Simple_free_token_main"
                            }
                        ]
                    }
                }
                "#
            "###,
        )
        .unwrap();
        let output = handle_import(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::Categorize, ::sbor::Encode, ::sbor::Decode, ::scrypto::LegacyDescribe)]
                #[sbor(custom_value_kind = "::scrypto::data::ScryptoCustomValueKind")]
                pub struct SimpleGlobalComponentRef {
                    component_address: ::scrypto::model::ComponentAddress,
                }
                impl SimpleGlobalComponentRef {
                    pub fn new() -> ::scrypto::model::ComponentAddress {
                        ::scrypto::runtime::Runtime::call_function(
                            ::scrypto::model::PackageAddress::try_from_hex("056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7").unwrap(),
                            "Simple",
                            "new",
                            args!()
                        )
                    }
                    pub fn free_token(&self) -> ::scrypto::model::Bucket {
                        ::scrypto::runtime::Runtime::call_method(
                            self.component_address,
                            "free_token",
                            args!()
                        )
                    }
                }
                impl From<::scrypto::model::ComponentAddress> for SimpleGlobalComponentRef {
                    fn from(component_address: ::scrypto::model::ComponentAddress) -> Self {
                        Self {
                            component_address
                        }
                    }
                }
                impl From<SimpleGlobalComponentRef> for ::scrypto::model::ComponentAddress {
                    fn from(a: SimpleGlobalComponentRef) -> ::scrypto::model::ComponentAddress {
                        a.component_address
                    }
                }
            },
        );
    }
}
