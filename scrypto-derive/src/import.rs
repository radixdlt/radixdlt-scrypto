use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::*;

use scrypto_abi as abi;
use scrypto_abi::Fields as SchemaFields;
use scrypto_abi::ScryptoTypeId;
use scrypto_abi::Type as SchemaType;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_import(input: TokenStream) -> Result<TokenStream> {
    trace!("Started processing import macro");

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
    let ident = format_ident!("{}", blueprint_name);
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
                    ::scrypto::core::Runtime::call_function(
                        ::scrypto::component::PackageAddress::try_from_hex(#package_address).unwrap(),
                        #blueprint_name,
                        #func_name,
                        ::scrypto::args!(#(#func_args),*)
                    )
                }
            });
        } else {
            fns.push(parse_quote! {
                pub fn #func_indent(&self #(, #func_args: #func_types)*) -> #func_output {
                    ::scrypto::core::Runtime::call_method(
                        self.component_address,
                        #func_name,
                        ::scrypto::args!(#(#func_args),*)
                    )
                }
            });
        }
    }

    let output = quote! {
        #(#structs)*

        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::scrypto::Describe)]
        pub struct #ident {
            component_address: ::scrypto::component::ComponentAddress,
        }

        impl #ident {
            #(#fns)*
        }

        impl From<::scrypto::component::ComponentAddress> for #ident {
            fn from(component_address: ::scrypto::component::ComponentAddress) -> Self {
                Self {
                    component_address
                }
            }
        }

        impl From<#ident> for ::scrypto::component::ComponentAddress {
            fn from(a: #ident) -> ::scrypto::component::ComponentAddress {
                a.component_address
            }
        }
    };
    trace!("Finished processing import macro");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("import!", &output);

    Ok(output)
}

fn get_native_type(ty: &SchemaType) -> Result<(Type, Vec<Item>)> {
    let mut structs = Vec::<Item>::new();

    let t: Type = match ty {
        // primitive types
        SchemaType::Unit => parse_quote! { () },
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
        // struct & enum
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
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::scrypto::Describe)]
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
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::scrypto::Describe)]
                        pub struct #ident (
                            #( pub #types ),*
                        );
                    });
                }
                SchemaFields::Unit => {
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::scrypto::Describe)]
                        pub struct #ident;
                    });
                }
            }

            parse_quote! { #ident }
        }
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
                #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::scrypto::Describe)]
                pub enum #ident {
                    #( #native_variants ),*
                }
            });

            parse_quote! { #ident }
        }
        // composite types
        SchemaType::Option { some_type } => {
            let (new_type, new_structs) = get_native_type(some_type)?;
            structs.extend(new_structs);

            parse_quote! { Option<#new_type> }
        }
        SchemaType::Tuple { element_types } => {
            let mut types: Vec<Type> = vec![];

            for element_type in element_types {
                let (new_type, new_structs) = get_native_type(element_type)?;
                types.push(new_type);
                structs.extend(new_structs);
            }

            parse_quote! { ( #(#types),* ) }
        }
        SchemaType::Array {
            element_type,
            length,
        } => {
            let (new_type, new_structs) = get_native_type(element_type)?;
            structs.extend(new_structs);

            let n = *length as usize;
            parse_quote! { [#new_type; #n] }
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
        // collection
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
        SchemaType::Any => {
            panic!("Any type not currently supported for importing.");
        }
        SchemaType::Custom { type_id, generics } => {
            // Copying the names to avoid cyclic dependency.
            let scrypto_type = ScryptoTypeId::from_id(*type_id).ok_or(Error::new(
                Span::call_site(),
                format!("Invalid custom type: {}", type_id),
            ))?;

            let canonical_name = match scrypto_type {
                // Global addresses types
                ScryptoTypeId::PackageAddress => "::scrypto::component::PackageAddress",
                ScryptoTypeId::ComponentAddress => "::scrypto::component::ComponentAddress",
                ScryptoTypeId::ResourceAddress => "::scrypto::resource::ResourceAddress",
                ScryptoTypeId::SystemAddress => "::scrypto::core::SystemAddress",
                // RE nodes types
                ScryptoTypeId::Component => "::scrypto::component::Component",
                ScryptoTypeId::KeyValueStore => "::scrypto::component::KeyValueStore",
                ScryptoTypeId::Bucket => "::scrypto::resource::Bucket",
                ScryptoTypeId::Proof => "::scrypto::resource::Proof",
                ScryptoTypeId::Vault => "::scrypto::resource::Vault",
                // Other interpreted types
                ScryptoTypeId::Expression => "::scrypto::core::Expression",
                ScryptoTypeId::Blob => "::scrypto::core::Blob",
                ScryptoTypeId::NonFungibleAddress => "::scrypto::resource::NonFungibleAddress",
                // Uninterpreted
                ScryptoTypeId::Hash => "::scrypto::crypto::Hash",
                ScryptoTypeId::EcdsaSecp256k1PublicKey => {
                    "::scrypto::crypto::EcdsaSecp256k1PublicKey"
                }
                ScryptoTypeId::EcdsaSecp256k1Signature => {
                    "::scrypto::crypto::EcdsaSecp256k1Signature"
                }
                ScryptoTypeId::EddsaEd25519PublicKey => "::scrypto::crypto::EddsaEd25519PublicKey",
                ScryptoTypeId::EddsaEd25519Signature => "::scrypto::crypto::EddsaEd25519Signature",
                ScryptoTypeId::Decimal => "::scrypto::math::Decimal",
                ScryptoTypeId::PreciseDecimal => "::scrypto::math::PreciseDecimal",
                ScryptoTypeId::NonFungibleId => "::scrypto::resource::NonFungibleId",
            };

            let ty: Type = parse_str(canonical_name).unwrap();
            if generics.is_empty() {
                parse_quote! { #ty }
            } else {
                let mut types = vec![];
                for g in generics {
                    let (t, v) = get_native_type(g)?;
                    types.push(t);
                    structs.extend(v);
                }
                parse_quote! { #ty<#(#types),*> }
            }
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
                                    "type": "Custom",
                                    "type_id": 129,
                                    "generics": []
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
                                    "type": "Custom",
                                    "type_id": 146,
                                    "generics": []
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
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::scrypto::Describe)]
                pub struct Simple {
                    component_address: ::scrypto::component::ComponentAddress,
                }
                impl Simple {
                    pub fn new() -> ::scrypto::component::ComponentAddress {
                        ::scrypto::core::Runtime::call_function(
                            ::scrypto::component::PackageAddress::try_from_hex("056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7").unwrap(),
                            "Simple",
                            "new",
                            ::scrypto::args!()
                        )
                    }
                    pub fn free_token(&self) -> ::scrypto::resource::Bucket {
                        ::scrypto::core::Runtime::call_method(
                            self.component_address,
                            "free_token",
                            ::scrypto::args!()
                        )
                    }
                }
                impl From<::scrypto::component::ComponentAddress> for Simple {
                    fn from(component_address: ::scrypto::component::ComponentAddress) -> Self {
                        Self {
                            component_address
                        }
                    }
                }
                impl From<Simple> for ::scrypto::component::ComponentAddress {
                    fn from(a: Simple) -> ::scrypto::component::ComponentAddress {
                        a.component_address
                    }
                }
            },
        );
    }
}
