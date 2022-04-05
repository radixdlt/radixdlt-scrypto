use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::*;

use sbor::describe as des;
use scrypto_abi as abi;

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

    let mut functions = Vec::<ItemFn>::new();
    for function in &blueprint.functions {
        trace!("Processing function: {:?}", function);

        let func_name = &function.name;
        let func_indent = format_ident!("{}", func_name);
        let mut func_types = Vec::<Type>::new();
        let mut func_args = Vec::<Ident>::new();

        for (i, input) in function.inputs.iter().enumerate() {
            let ident = format_ident!("arg{}", i);
            let (new_type, new_structs) = get_native_type(input)?;
            func_args.push(parse_quote! { #ident });
            func_types.push(parse_quote! { #new_type });
            structs.extend(new_structs);
        }
        let (func_output, new_structs) = get_native_type(&function.output)?;
        structs.extend(new_structs);

        functions.push(parse_quote! {
            pub fn #func_indent(#(#func_args: #func_types),*) -> #func_output {
                let rtn = ::scrypto::core::Runtime::call_function(
                    ::scrypto::component::PackageAddress::from_str(#package_address).unwrap(),
                    #blueprint_name,
                    #func_name,
                    ::scrypto::args!(#(#func_args),*)
                );
                ::scrypto::buffer::scrypto_decode(&rtn).unwrap()
            }
        });
    }

    let mut methods = Vec::<ItemFn>::new();
    for method in &blueprint.methods {
        trace!("Processing method: {:?}", method);

        let method_name = &method.name;
        let method_indent = format_ident!("{}", method_name);
        let mut method_types = Vec::<Type>::new();
        let mut method_args = Vec::<Ident>::new();

        for (i, input) in method.inputs.iter().enumerate() {
            let ident = format_ident!("arg{}", i);
            let (new_type, new_structs) = get_native_type(input)?;
            method_args.push(parse_quote! { #ident });
            method_types.push(parse_quote! { #new_type });
            structs.extend(new_structs);
        }
        let (method_output, new_structs) = get_native_type(&method.output)?;
        structs.extend(new_structs);

        let m = parse_quote! {
            pub fn #method_indent(&self #(, #method_args: #method_types)*) -> #method_output {
                let rtn = ::scrypto::core::Runtime::call_method(
                    self.component_address,
                    #method_name,
                    ::scrypto::args!(#(#method_args),*)
                );
                ::scrypto::buffer::scrypto_decode(&rtn).unwrap()
            }
        };
        methods.push(m);
    }

    let output = quote! {
        #(#structs)*

        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
        pub struct #ident {
            component_address: ::scrypto::component::ComponentAddress,
        }

        impl #ident {
            #(#functions)*

            #(#methods)*
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
                #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
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
        des::Type::Custom { name, generics } => {
            // Copying the names to avoid cyclic dependency.

            let canonical_name = match name.as_str() {
                "PackageAddress" => "::scrypto::component::PackageAddress",
                "ComponentAddress" => "::scrypto::component::ComponentAddress",
                "LazyMap" => "::scrypto::component::LazyMap",
                "Hash" => "::scrypto::crypto::Hash",
                "EcdsaPublicKey" => "::scrypto::crypto::EcdsaPublicKey",
                "EcdsaSignature" => "::scrypto::crypto::EcdsaSignature",
                "Decimal" => "::scrypto::math::Decimal",
                "Bucket" => "::scrypto::resource::Bucket",
                "Proof" => "::scrypto::resource::Proof",
                "Vault" => "::scrypto::resource::Vault",
                "NonFungibleId" => "::scrypto::resource::NonFungibleId",
                "NonFungibleAddress" => "::scrypto::resource::NonFungibleAddress",
                "ResourceAddress" => "::scrypto::resource::ResourceAddress",
                _ => {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("Invalid custom type: {}", name),
                    ));
                }
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
                    "functions": [
                        {
                            "name": "new",
                            "inputs": [],
                            "output": {
                                "type": "Custom",
                                "name": "ComponentAddress",
                                "generics": []
                            }
                        }
                    ],
                    "methods": [
                        {
                            "name": "free_token",
                            "mutability": "Mutable",
                            "inputs": [
                            ],
                            "output": {
                                "type": "Custom",
                                "name": "Bucket",
                                "generics": []
                            }
                        }
                    ]
                }
                "#
            "###,
        )
        .unwrap();
        let output = handle_import(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                pub struct Simple {
                    component_address: ::scrypto::component::ComponentAddress,
                }
                impl Simple {
                    pub fn new() -> ::scrypto::component::ComponentAddress {
                        let rtn = ::scrypto::core::Runtime::call_function(
                            ::scrypto::component::PackageAddress::from_str("056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7").unwrap(),
                            "Simple",
                            "new",
                            ::scrypto::args!()
                        );
                        ::scrypto::buffer::scrypto_decode(&rtn).unwrap()
                    }
                    pub fn free_token(&self) -> ::scrypto::resource::Bucket {
                        let rtn = ::scrypto::core::Runtime::call_method(
                            self.component_address,
                            "free_token",
                            ::scrypto::args!()
                        );
                        ::scrypto::buffer::scrypto_decode(&rtn).unwrap()
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
