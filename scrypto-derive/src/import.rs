use std::fs;
use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::*;

use scrypto_abi as abi;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_import(input: TokenStream) -> TokenStream {
    trace!("handle_import() starts");
    let span = Span::call_site();

    let path = parse2::<LitStr>(input)
        .expect("Unable to parse input")
        .value();
    let content = fs::read_to_string(path).expect("Unable to load ABI");
    let blueprint: abi::Blueprint =
        serde_json::from_str(content.as_str()).expect("Unable to parse ABI");
    trace!("ABI: {:?}", blueprint);

    let mut items: Vec<Item> = vec![];
    let mut implementations: Vec<ItemImpl> = vec![];

    let package = blueprint.package;
    let name = blueprint.blueprint;
    let ident = Ident::new(name.as_str(), span);
    trace!("Blueprint: {}", quote! { #ident });

    let structure: Item = parse_quote! {
        pub struct #ident {
            address: ::scrypto::types::Address
        }
    };
    items.push(structure);

    let mut functions = Vec::<ItemFn>::new();
    functions.push(parse_quote! {
        pub fn from_address(address: ::scrypto::types::Address) -> Self {
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
            match input {
                _ => {
                    let ident = format_ident!("arg{}", i);
                    let (new_type, new_items) = get_native_type(input);
                    func_args.push(parse_quote! { #ident });
                    func_inputs.push(parse_quote! { #ident: #new_type });
                    items.extend(new_items);
                }
            }
            if i < function.inputs.len() - 1 {
                func_inputs.push_punct(Comma(span));
            }
        }
        let (func_output, new_items) = get_native_type(&function.output);
        items.extend(new_items);

        functions.push(parse_quote! {
            pub fn #func_indent(#func_inputs) -> #func_output {
                let package = ::scrypto::types::Address::from_str(#package).unwrap();
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
            match input {
                _ => {
                    let ident = format_ident!("arg{}", i);
                    let (new_type, new_items) = get_native_type(input);
                    method_args.push(parse_quote! { #ident });
                    method_inputs.push(parse_quote! { #ident: #new_type });
                    items.extend(new_items);
                }
            }
            if i < method.inputs.len() - 1 {
                method_inputs.push_punct(Comma(span));
            }
        }
        let (method_output, new_items) = get_native_type(&method.output);
        items.extend(new_items);

        let m = match method.mutability {
            abi::Mutability::Immutable => {
                parse_quote! {
                    pub fn #method_indent(&self, #method_inputs) -> #method_output {
                        let component = ::scrypto::constructs::Component::from(self.address);
                        component.call(#method_name, ::scrypto::args!(#(#method_args),*))
                    }
                }
            }
            abi::Mutability::Mutable => {
                parse_quote! {
                    pub fn #method_indent(&mut self, #method_inputs) -> #method_output {
                        let component = ::scrypto::constructs::Component::from(self.address);
                        component.call(#method_name, ::scrypto::args!(#(#method_args),*))
                    }
                }
            }
        };
        functions.push(m);
    }

    let implementation = parse_quote! {
        impl #ident {
            #(#functions)*
        }
    };
    trace!("Implementation: {}", quote! { #implementation });
    implementations.push(implementation);

    let output = quote! {
         #(#items)*

         #(#implementations)*
    };
    trace!("handle_import() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("import!", &output);

    output.into()
}

fn get_native_type(ty: &sbor::model::Type) -> (Type, Vec<Item>) {
    let mut items = Vec::<Item>::new();

    let t: Type = match ty {
        sbor::model::Type::Unit => parse_quote! { () },
        sbor::model::Type::Bool => parse_quote! { bool },
        sbor::model::Type::I8 => parse_quote! { i8 },
        sbor::model::Type::I16 => parse_quote! { i16 },
        sbor::model::Type::I32 => parse_quote! { i32 },
        sbor::model::Type::I64 => parse_quote! { i64 },
        sbor::model::Type::I128 => parse_quote! { i128 },
        sbor::model::Type::U8 => parse_quote! { u8 },
        sbor::model::Type::U16 => parse_quote! { u16 },
        sbor::model::Type::U32 => parse_quote! { u32 },
        sbor::model::Type::U64 => parse_quote! { u64 },
        sbor::model::Type::U128 => parse_quote! { u128 },
        sbor::model::Type::String => parse_quote! { String },
        sbor::model::Type::H256 => parse_quote! { ::scrypto::types::H256 },
        sbor::model::Type::U256 => parse_quote! { ::scrypto::types::U256 },
        sbor::model::Type::Address => parse_quote! { ::scrypto::types::Address },
        sbor::model::Type::BID => parse_quote! { ::scrypto::types::BID },
        sbor::model::Type::RID => parse_quote! { ::scrypto::types::RID },
        sbor::model::Type::Option { value } => {
            let (new_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { Option<#new_type> }
        }
        sbor::model::Type::Box { value } => {
            let (new_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { Box<#new_type> }
        }
        sbor::model::Type::Struct { name, fields } => {
            let ident = format_ident!("{}", name);

            match fields {
                sbor::model::Fields::Named { named } => {
                    let names: Vec<Ident> =
                        named.iter().map(|k| format_ident!("{}", k.0)).collect();
                    let mut types: Vec<Type> = vec![];
                    for (_, v) in named {
                        let (new_type, new_items) = get_native_type(v);
                        types.push(new_type);
                        items.extend(new_items);
                    }
                    items.push(parse_quote! {
                        #[derive(Debug, ::sbor::Encode, ::sbor::Decode)]
                        pub struct #ident {
                            #( pub #names : #types, )*
                        }
                    });
                }
                _ => {
                    todo!("Add support for non-named fields")
                }
            }

            parse_quote! { #ident }
        }
        sbor::model::Type::Tuple { elements } => {
            let mut types: Vec<Type> = vec![];

            for element in elements {
                let (new_type, new_items) = get_native_type(element);
                types.push(new_type);
                items.extend(new_items);
            }

            parse_quote! { ( #(#types),* ) }
        }
        sbor::model::Type::Array { element, length } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            let n = *length as usize;
            parse_quote! { [#new_type; #n] }
        }
        sbor::model::Type::Enum { name, variants } => {
            let ident = format_ident!("{}", name);
            let mut native_variants = Vec::<Variant>::new();

            for variant in variants {
                let v_ident = format_ident!("{}", variant.name);

                match &variant.fields {
                    sbor::model::Fields::Named { named } => {
                        let mut names: Vec<Ident> = vec![];
                        let mut types: Vec<Type> = vec![];
                        for (n, v) in named {
                            names.push(format_ident!("{}", n));
                            let (new_type, new_items) = get_native_type(&v);
                            types.push(new_type);
                            items.extend(new_items);
                        }
                        native_variants.push(parse_quote! {
                            #v_ident {
                                #(#names: #types),*
                            }
                        });
                    }
                    sbor::model::Fields::Unnamed { unnamed } => {
                        let mut types: Vec<Type> = vec![];
                        for v in unnamed {
                            let (new_type, new_items) = get_native_type(&v);
                            types.push(new_type);
                            items.extend(new_items);
                        }
                        native_variants.push(parse_quote! {
                            #v_ident ( #(#types),* )
                        });
                    }
                    sbor::model::Fields::Unit => {
                        native_variants.push(parse_quote! {
                            #v_ident
                        });
                    }
                };
            }

            items.push(parse_quote! {
                #[derive(Debug, ::sbor::Encode, ::sbor::Decode)]
                pub enum #ident {
                    #( #native_variants ),*
                }
            });

            parse_quote! { #ident }
        }
        sbor::model::Type::Vec { element } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            parse_quote! { Vec<#new_type> }
        }
        sbor::model::Type::TreeSet { element } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            parse_quote! { BTreeSet<#new_type> }
        }
        sbor::model::Type::TreeMap { key, value } => {
            let (key_type, new_items) = get_native_type(key);
            items.extend(new_items);
            let (value_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { BTreeMap<#key_type, #value_type> }
        }
        sbor::model::Type::HashSet { element } => {
            let (new_type, new_items) = get_native_type(element);
            items.extend(new_items);

            parse_quote! { HashSet<#new_type> }
        }
        sbor::model::Type::HashMap { key, value } => {
            let (key_type, new_items) = get_native_type(key);
            items.extend(new_items);
            let (value_type, new_items) = get_native_type(value);
            items.extend(new_items);

            parse_quote! { HashMap<#key_type, #value_type> }
        }
        sbor::model::Type::SystemType { name } => {
            if !name.starts_with("::scrypto::") {
                panic!("Invalid system type: {}", name);
            }

            let path = TokenStream::from_str(name.as_str())
                .expect(format!("Invalid system type: {}", name).as_str());
            parse_quote! { #path }
        }
    };

    (t, items)
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
    fn test_import_stateful() {
        let input = TokenStream::from_str("\"../scrypto-derive/tests/abi_stateful.json\"").unwrap();
        let output = handle_import(input);

        assert_code_eq(
            output,
            quote! {
                pub struct Sample {
                    address: ::scrypto::types::Address
                }
                #[derive(Debug, ::sbor::Encode, ::sbor::Decode)]
                pub struct Floor {
                    pub x: u32,
                    pub y: u32,
                }
                #[derive(Debug, ::sbor::Encode, ::sbor::Decode)]
                pub enum Hello {
                    A { x: u32 },
                    B(u32),
                    C
                }
                impl Sample {
                    pub fn from_address(address: ::scrypto::types::Address) -> Self {
                        Self { address }
                    }
                    pub fn calculate_volume(
                        &self,
                        arg0: Floor,
                        arg1: (u8, u16),
                        arg2: Vec<String>,
                        arg3: u32,
                        arg4: Hello,
                        arg5: [String; 2usize]
                    ) -> u32 {
                        let component = ::scrypto::constructs::Component::from(self.address);
                        component.call(
                            "calculate_volume",
                            ::scrypto::args!(arg0, arg1, arg2, arg3, arg4, arg5)
                        )
                    }
                }
            },
        );
    }

    #[test]
    fn test_import_stateless() {
        let input =
            TokenStream::from_str("\"../scrypto-derive/tests/abi_stateless.json\"").unwrap();
        let output = handle_import(input);

        assert_code_eq(
            output,
            quote! {
                pub struct Sample {
                    address: ::scrypto::types::Address
                }
                impl Sample {
                    pub fn from_address(address: ::scrypto::types::Address) -> Self {
                        Self { address }
                    }
                    pub fn stateless_func() -> u32 {
                        let package = ::scrypto::types::Address::from_str(
                            "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7"
                        )
                        .unwrap();
                        let blueprint = ::scrypto::constructs::Blueprint::from(package, "Sample");
                        blueprint.call("stateless_func", ::scrypto::args!())
                    }
                }
            },
        );
    }

    #[test]
    fn test_import_system_types() {
        let input =
            TokenStream::from_str("\"../scrypto-derive/tests/abi_system_types.json\"").unwrap();
        let output = handle_import(input);

        assert_code_eq(
            output,
            quote! {
                pub struct Sample {
                    address: ::scrypto::types::Address
                }
                impl Sample {
                    pub fn from_address(address: ::scrypto::types::Address) -> Self {
                        Self { address }
                    }
                    pub fn test_system_types(arg0: ::scrypto::resource::Tokens) -> ::scrypto::resource::BadgesRef {
                        let package = ::scrypto::types::Address::from_str(
                            "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7"
                        )
                        .unwrap();
                        let blueprint = ::scrypto::constructs::Blueprint::from(package, "Sample");
                        blueprint.call("test_system_types", ::scrypto::args!(arg0))
                    }
                }
            },
        );
    }
}
