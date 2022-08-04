use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::*;

use sbor::describe as des;
use scrypto_abi as abi;
use scrypto_abi::ScryptoType;

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
            sbor::Type::Struct {
                name: _,
                fields: sbor::describe::Fields::Named { named },
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
                        ::scrypto::component::PackageAddress::from_str(#package_address).unwrap(),
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

        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
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

fn get_native_type(ty: &des::Type) -> Result<(Type, Vec<Item>)> {
    let mut structs = Vec::<Item>::new();

    let t: Type = match ty {
        // primitive types
        des::Type::Unit => parse_quote! { () },
        des::Type::Bool => parse_quote! { bool },
        des::Type::I8 => parse_quote! { i8 },
        des::Type::I16 => parse_quote! { i16 },
        des::Type::I32 => parse_quote! { i32 },
        des::Type::I64 => parse_quote! { i64 },
        des::Type::I128 => parse_quote! { i128 },
        des::Type::U8 => parse_quote! { u8 },
        des::Type::U16 => parse_quote! { u16 },
        des::Type::U32 => parse_quote! { u32 },
        des::Type::U64 => parse_quote! { u64 },
        des::Type::U128 => parse_quote! { u128 },
        des::Type::String => parse_quote! { String },
        // struct & enum
        des::Type::Struct { name, fields } => {
            let ident = format_ident!("{}", name);

            match fields {
                des::Fields::Named { named } => {
                    let names: Vec<Ident> =
                        named.iter().map(|k| format_ident!("{}", k.0)).collect();
                    let mut types: Vec<Type> = vec![];
                    for (_, v) in named {
                        let (new_type, new_structs) = get_native_type(v)?;
                        types.push(new_type);
                        structs.extend(new_structs);
                    }
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                        pub struct #ident {
                            #( pub #names : #types, )*
                        }
                    });
                }
                des::Fields::Unnamed { unnamed } => {
                    let mut types: Vec<Type> = vec![];
                    for v in unnamed {
                        let (new_type, new_structs) = get_native_type(v)?;
                        types.push(new_type);
                        structs.extend(new_structs);
                    }
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                        pub struct #ident (
                            #( pub #types ),*
                        )
                    });
                }
                des::Fields::Unit => {
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                        pub struct #ident;
                    });
                }
            }

            parse_quote! { #ident }
        }
        des::Type::Enum { name, variants } => {
            let ident = format_ident!("{}", name);
            let mut native_variants = Vec::<Variant>::new();

            for variant in variants {
                let v_ident = format_ident!("{}", variant.name);

                match &variant.fields {
                    des::Fields::Named { named } => {
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
                    des::Fields::Unnamed { unnamed } => {
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
                    des::Fields::Unit => {
                        native_variants.push(parse_quote! {
                            #v_ident
                        });
                    }
                };
            }

            structs.push(parse_quote! {
                #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                pub enum #ident {
                    #( #native_variants ),*
                }
            });

            parse_quote! { #ident }
        }
        // composite types
        des::Type::Option { value } => {
            let (new_type, new_structs) = get_native_type(value)?;
            structs.extend(new_structs);

            parse_quote! { Option<#new_type> }
        }
        des::Type::Tuple { elements } => {
            let mut types: Vec<Type> = vec![];

            for element in elements {
                let (new_type, new_structs) = get_native_type(element)?;
                types.push(new_type);
                structs.extend(new_structs);
            }

            parse_quote! { ( #(#types),* ) }
        }
        des::Type::Array { element, length } => {
            let (new_type, new_structs) = get_native_type(element)?;
            structs.extend(new_structs);

            let n = *length as usize;
            parse_quote! { [#new_type; #n] }
        }
        des::Type::Result { okay, error } => {
            let (okay_type, new_structs) = get_native_type(okay)?;
            structs.extend(new_structs);
            let (error_type, new_structs) = get_native_type(error)?;
            structs.extend(new_structs);

            parse_quote! { Result<#okay_type, #error_type> }
        }
        // collection
        des::Type::Vec { element } => {
            let (new_type, new_structs) = get_native_type(element)?;
            structs.extend(new_structs);

            parse_quote! { Vec<#new_type> }
        }
        des::Type::TreeSet { element } => {
            let (new_type, new_structs) = get_native_type(element)?;
            structs.extend(new_structs);

            parse_quote! { BTreeSet<#new_type> }
        }
        des::Type::TreeMap { key, value } => {
            let (key_type, new_structs) = get_native_type(key)?;
            structs.extend(new_structs);
            let (value_type, new_structs) = get_native_type(value)?;
            structs.extend(new_structs);

            parse_quote! { BTreeMap<#key_type, #value_type> }
        }
        des::Type::HashSet { element } => {
            let (new_type, new_structs) = get_native_type(element)?;
            structs.extend(new_structs);

            parse_quote! { HashSet<#new_type> }
        }
        des::Type::HashMap { key, value } => {
            let (key_type, new_structs) = get_native_type(key)?;
            structs.extend(new_structs);
            let (value_type, new_structs) = get_native_type(value)?;
            structs.extend(new_structs);

            parse_quote! { HashMap<#key_type, #value_type> }
        }
        des::Type::Any => {
            panic!("Any type not currently supported for importing.");
        }
        des::Type::Custom { type_id, generics } => {
            // Copying the names to avoid cyclic dependency.
            let scrypto_type = ScryptoType::from_id(*type_id).ok_or(Error::new(
                Span::call_site(),
                format!("Invalid custom type: {}", type_id),
            ))?;

            let canonical_name = match scrypto_type {
                ScryptoType::PackageAddress => "::scrypto::component::PackageAddress",
                ScryptoType::ComponentAddress => "::scrypto::component::ComponentAddress",
                ScryptoType::Component => "::scrypto::component::Component",
                ScryptoType::KeyValueStore => "::scrypto::component::KeyValueStore",
                ScryptoType::Hash => "::scrypto::crypto::Hash",
                ScryptoType::EcdsaPublicKey => "::scrypto::crypto::EcdsaPublicKey",
                ScryptoType::EcdsaSignature => "::scrypto::crypto::EcdsaSignature",
                ScryptoType::Ed25519PublicKey => "::scrypto::crypto::Ed25519PublicKey",
                ScryptoType::Ed25519Signature => "::scrypto::crypto::Ed25519Signature",
                ScryptoType::Decimal => "::scrypto::math::Decimal",
                ScryptoType::U8 => "::scrypto::math::U8",
                ScryptoType::U16 => "::scrypto::math::U16",
                ScryptoType::U32 => "::scrypto::math::U32",
                ScryptoType::U64 => "::scrypto::math::U64",
                ScryptoType::U128 => "::scrypto::math::U128",
                ScryptoType::U256 => "::scrypto::math::U256",
                ScryptoType::U384 => "::scrypto::math::U384",
                ScryptoType::U512 => "::scrypto::math::U512",
                ScryptoType::I8 => "::scrypto::math::I8",
                ScryptoType::I16 => "::scrypto::math::I16",
                ScryptoType::I32 => "::scrypto::math::I32",
                ScryptoType::I64 => "::scrypto::math::I64",
                ScryptoType::I128 => "::scrypto::math::I128",
                ScryptoType::I256 => "::scrypto::math::I256",
                ScryptoType::I384 => "::scrypto::math::I384",
                ScryptoType::I512 => "::scrypto::math::I512",
                ScryptoType::Bucket => "::scrypto::resource::Bucket",
                ScryptoType::Proof => "::scrypto::resource::Proof",
                ScryptoType::Vault => "::scrypto::resource::Vault",
                ScryptoType::NonFungibleId => "::scrypto::resource::NonFungibleId",
                ScryptoType::NonFungibleAddress => "::scrypto::resource::NonFungibleAddress",
                ScryptoType::ResourceAddress => "::scrypto::resource::ResourceAddress",
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
                                    "type_id": 177,
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
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                pub struct Simple {
                    component_address: ::scrypto::component::ComponentAddress,
                }
                impl Simple {
                    pub fn new() -> ::scrypto::component::ComponentAddress {
                        ::scrypto::core::Runtime::call_function(
                            ::scrypto::component::PackageAddress::from_str("056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7").unwrap(),
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
