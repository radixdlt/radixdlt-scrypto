use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::*;

use sbor::describe as des;
use scrypto_abi as abi;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_import(input: TokenStream) -> TokenStream {
    trace!("Started processing import macro");
    let span = Span::call_site();

    let content = parse2::<LitStr>(input)
        .expect("Unable to parse input")
        .value();
    let blueprint: abi::Blueprint =
        serde_json::from_str(content.as_str()).expect("Unable to parse ABI");
    trace!("Parsed ABI: {:?}", blueprint);

    let mut items: Vec<Item> = vec![];
    let mut implementations: Vec<ItemImpl> = vec![];

    let package = blueprint.package;
    let name = blueprint.blueprint;
    let ident = Ident::new(name.as_str(), span);
    trace!("Blueprint name: {}", name);

    let structure: Item = parse_quote! {
        pub struct #ident {
            address: ::scrypto::types::Address
        }
    };
    items.push(structure);

    let mut functions = Vec::<ItemFn>::new();
    functions.push(parse_quote! {
        pub fn instance(address: ::scrypto::types::Address) -> Self {
            Self {
                address
            }
        }
    });

    for function in &blueprint.functions {
        trace!("Processing function: {:?}", function);

        let func_name = &function.name;
        let func_indent = Ident::new(func_name.as_str(), span);
        let mut func_inputs = Punctuated::<FnArg, Comma>::new();
        let mut func_args = Vec::<Ident>::new();

        for (i, input) in function.inputs.iter().enumerate() {
            let ident = format_ident!("arg{}", i);
            let (new_type, new_items) = get_native_type(input);
            func_args.push(parse_quote! { #ident });
            func_inputs.push(parse_quote! { #ident: #new_type });
            items.extend(new_items);

            if i < function.inputs.len() - 1 {
                func_inputs.push_punct(Comma(span));
            }
        }
        let (func_output, new_items) = get_native_type(&function.output);
        items.extend(new_items);

        functions.push(parse_quote! {
            pub fn #func_indent(#func_inputs) -> #func_output {
                let package = ::scrypto::utils::unwrap_light(
                    ::scrypto::types::Address::from_str(#package)
                );
                let blueprint = ::scrypto::constructs::Blueprint::from(package, #name);
                blueprint.call(#func_name, ::scrypto::args!(#(#func_args),*))
            }
        });
    }

    for method in &blueprint.methods {
        trace!("Processing method: {:?}", method);

        let method_name = &method.name;
        let method_indent = Ident::new(method_name.as_str(), span);
        let mut method_inputs = Punctuated::<FnArg, Comma>::new();
        let mut method_args = Vec::<Ident>::new();

        for (i, input) in method.inputs.iter().enumerate() {
            let ident = format_ident!("arg{}", i);
            let (new_type, new_items) = get_native_type(input);
            method_args.push(parse_quote! { #ident });
            method_inputs.push(parse_quote! { #ident: #new_type });
            items.extend(new_items);

            if i < method.inputs.len() - 1 {
                method_inputs.push_punct(Comma(span));
            }
        }
        let (method_output, new_items) = get_native_type(&method.output);
        items.extend(new_items);

        let m = parse_quote! {
            pub fn #method_indent(&self, #method_inputs) -> #method_output {
                let component = ::scrypto::constructs::Component::from(self.address);
                component.call(#method_name, ::scrypto::args!(#(#method_args),*))
            }
        };
        functions.push(m);
    }

    let implementation = parse_quote! {
        impl #ident {
            #(#functions)*
        }
    };
    trace!("Generated impl: {}", quote! { #implementation });
    implementations.push(implementation);

    let output = quote! {
         #(#items)*

         #(#implementations)*
    };
    trace!("Finished processing import macro");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("import!", &output);

    output
}

fn get_native_type(ty: &des::Type) -> (Type, Vec<Item>) {
    let mut items = Vec::<Item>::new();

    let t: Type = match ty {
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
        des::Type::Option { value } => {
            let (new_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { Option<#new_type> }
        }
        des::Type::Box { value } => {
            let (new_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { Box<#new_type> }
        }
        des::Type::Struct { name, fields } => {
            let ident = format_ident!("{}", name);

            match fields {
                des::Fields::Named { named } => {
                    let names: Vec<Ident> =
                        named.iter().map(|k| format_ident!("{}", k.0)).collect();
                    let mut types: Vec<Type> = vec![];
                    for (_, v) in named {
                        let (new_type, new_items) = get_native_type(v);
                        types.push(new_type);
                        items.extend(new_items);
                    }
                    items.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                        pub struct #ident {
                            #( pub #names : #types, )*
                        }
                    });
                }
                des::Fields::Unnamed { unnamed } => {
                    let mut types: Vec<Type> = vec![];
                    for v in unnamed {
                        let (new_type, new_items) = get_native_type(v);
                        types.push(new_type);
                        items.extend(new_items);
                    }
                    items.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                        pub struct #ident (
                            #( pub #types ),*
                        )
                    });
                }
                des::Fields::Unit => {
                    items.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                        pub struct #ident;
                    });
                }
            }

            parse_quote! { #ident }
        }
        des::Type::Tuple { elements } => {
            let mut types: Vec<Type> = vec![];

            for element in elements {
                let (new_type, new_items) = get_native_type(element);
                types.push(new_type);
                items.extend(new_items);
            }

            parse_quote! { ( #(#types),* ) }
        }
        des::Type::Array { element, length } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            let n = *length as usize;
            parse_quote! { [#new_type; #n] }
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
                            let (new_type, new_items) = get_native_type(v);
                            types.push(new_type);
                            items.extend(new_items);
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
                            let (new_type, new_items) = get_native_type(v);
                            types.push(new_type);
                            items.extend(new_items);
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

            items.push(parse_quote! {
                #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                pub enum #ident {
                    #( #native_variants ),*
                }
            });

            parse_quote! { #ident }
        }
        des::Type::Result { okay, error } => {
            let (okay_type, new_items) = get_native_type(okay);
            items.extend(new_items);
            let (error_type, new_items) = get_native_type(error);
            items.extend(new_items);

            parse_quote! { Result<#okay_type, #error_type> }
        }
        des::Type::Vec { element } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            parse_quote! { Vec<#new_type> }
        }
        des::Type::TreeSet { element } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            parse_quote! { BTreeSet<#new_type> }
        }
        des::Type::TreeMap { key, value } => {
            let (key_type, new_items) = get_native_type(key);
            items.extend(new_items);
            let (value_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { BTreeMap<#key_type, #value_type> }
        }
        des::Type::HashSet { element } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            parse_quote! { HashSet<#new_type> }
        }
        des::Type::HashMap { key, value } => {
            let (key_type, new_items) = get_native_type(key);
            items.extend(new_items);
            let (value_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { HashMap<#key_type, #value_type> }
        }
        des::Type::Custom { name } => match name.as_str() {
            "scrypto::Address" => parse_quote! { ::scrypto::types::Address },
            "scrypto::Amount" => parse_quote! { ::scrypto::types::Amount },
            "scrypto::H256" => parse_quote! { ::scrypto::types::H256 },
            "scrypto::BID" => parse_quote! { ::scrypto::types::BID },
            "scrypto::RID" => parse_quote! { ::scrypto::types::RID },
            "scrypto::SID" => parse_quote! { ::scrypto::types::SID },
            "scrypto::VID" => parse_quote! { ::scrypto::types::VID },

            "scrypto::Package" => parse_quote! { ::scrypto::constructs::Package },
            "scrypto::Blueprint" => parse_quote! { ::scrypto::constructs::Blueprint },
            "scrypto::Component" => parse_quote! { ::scrypto::constructs::Component },
            "scrypto::Storage" => parse_quote! { ::scrypto::constructs::Storage },

            "scrypto::Resource" => parse_quote! { ::scrypto::resource::Resource },
            "scrypto::Bucket" => parse_quote! { ::scrypto::resource::Bucket },
            "scrypto::BucketRef" => parse_quote! { ::scrypto::resource::BucketRef },
            "scrypto::Vault" => parse_quote! { ::scrypto::resource::Vault },

            _ => panic!("Invalid custom type: {}", name),
        },
    };

    (t, items)
}
