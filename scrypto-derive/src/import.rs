use std::fs;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
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
    let span = Span::call_site();

    let path_lit = parse_macro_input!(input as LitStr);
    let path = path_lit.value();
    let abi = fs::read_to_string(path).expect("Unable to load Abi");
    let component: abi::Component =
        serde_json::from_str(abi.as_str()).expect("Unable to parse Abi");
    trace!("ABI: {:?}", component);

    let mut structures: Vec<ItemStruct> = vec![];
    let mut implementations: Vec<ItemImpl> = vec![];

    let ident = Ident::new(component.name.as_str(), span);

    let structure: ItemStruct = parse_quote! {
        pub struct #ident {
            address: scrypto::types::Address
        }
    };

    let mut functions = Vec::<ItemFn>::new();
    functions.push(parse_quote! {
        pub fn from_address(address: scrypto::types::Address) -> Self {
            Self {
                address
            }
        }
    });

    for method in &component.methods {
        let func_indent = Ident::new(method.name.as_str(), span);
        let mut func_inputs = Punctuated::<FnArg, Comma>::new();

        match method.kind {
            abi::MethodKind::Stateful => match method.mutability {
                abi::Mutability::Immutable => func_inputs.push(parse_quote! { &self }),
                abi::Mutability::Mutable => func_inputs.push(parse_quote! { &mut self }),
            },
            _ => {}
        }

        for (i, input) in method.inputs.iter().enumerate() {
            match input {
                _ => {
                    let ident = format_ident!("arg{}", i);
                    let (new_type, new_structures) = get_native_type(input);
                    func_inputs.push(parse_quote! { #ident: #new_type });
                    structures.extend(new_structures);
                }
            }
            if i < method.inputs.len() - 1 {
                func_inputs.push_punct(Comma(span));
            }
        }
        let (func_output, new_structures) = get_native_type(&method.output);
        structures.extend(new_structures);

        functions.push(parse_quote! {
            pub fn #func_indent(#func_inputs) -> #func_output {
                todo!()
            }
        });
    }

    structures.push(structure);
    implementations.push(parse_quote! {
        impl #ident {
            #(#functions)*
        }
    });

    let output = quote! {
         #(#structures)*

         #(#implementations)*
    };

    trace!("Auto-generated code:\n\n{}\n", output);
    output.into()
}

fn get_native_type(ty: &abi::Type) -> (Type, Vec<ItemStruct>) {
    let mut structures = Vec::<ItemStruct>::new();

    let t: Type = match ty {
        abi::Type::Void => parse_quote! { () },
        abi::Type::Bool => parse_quote! { bool },
        abi::Type::I8 => parse_quote! { i8 },
        abi::Type::I16 => parse_quote! { i16 },
        abi::Type::I32 => parse_quote! { i32 },
        abi::Type::I64 => parse_quote! { i64 },
        abi::Type::I128 => parse_quote! { u128 },
        abi::Type::U8 => parse_quote! { u8 },
        abi::Type::U16 => parse_quote! { u16 },
        abi::Type::U32 => parse_quote! { u32 },
        abi::Type::U64 => parse_quote! { u64 },
        abi::Type::U128 => parse_quote! { u128 },
        abi::Type::String => parse_quote! { String },
        abi::Type::Option { value } => {
            let (new_type, new_structures) = get_native_type(value);
            structures.extend(new_structures);

            parse_quote! { Option<#new_type> }
        }
        abi::Type::Struct { name, attributes } => {
            let ident = Ident::new(name.as_str(), Span::call_site());

            let attrs: Vec<Ident> = attributes
                .keys()
                .map(|k| Ident::new(k.as_str(), Span::call_site()))
                .collect();
            let mut types: Vec<Type> = vec![];
            for v in attributes.values() {
                let (new_type, new_structures) = get_native_type(v);
                types.push(new_type);
                structures.extend(new_structures);
            }
            structures.push(parse_quote! {
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                pub struct #ident {
                    #( pub #attrs : #types, )*
                }
            });

            parse_quote! { #ident }
        }
        abi::Type::Tuple { elements } => {
            let mut types: Vec<Type> = vec![];

            for element in elements {
                let (new_type, new_structures) = get_native_type(element);
                types.push(new_type);
                structures.extend(new_structures);
            }

            parse_quote! { ( #(#types),* ) }
        }
        abi::Type::Array { base } => {
            let (new_type, new_structures) = get_native_type(base);
            structures.extend(new_structures);

            parse_quote! { Vec<#new_type> }
        }
    };

    (t, structures)
}
