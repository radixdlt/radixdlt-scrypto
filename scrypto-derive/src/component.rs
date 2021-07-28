use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::token::Comma;
use syn::*;

use crate::ast;
use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_component(input: TokenStream) -> TokenStream {
    trace!("handle_component() begins");

    // parse component struct and impl
    let com: ast::Component = parse2(input).expect("Unable to parse input");
    let com_strut = &com.structure;
    let com_impl = &com.implementation;
    let com_ident = &com_strut.ident;
    let com_items = &com_impl.items;
    let com_name = com_ident.to_string();
    trace!("Processing component: {}", com_name);

    trace!("Generating dispatcher function...");
    let dispatcher_ident = format_ident!("{}_main", com_ident);
    let (arm_guards, arm_bodies) = generate_dispatcher(&com_ident, com_items);

    trace!("Generating ABI function...");
    let abi_ident = format_ident!("{}_abi", com_ident);
    let abi_methods = generate_abi(&com_name, com_items);

    trace!("Generating stubs...");
    let stub_ident = format_ident!("{}Stub", com_ident);
    let stub_items = generate_stub(&com_ident, com_items);

    let output = quote! {
        #[derive(Debug, scrypto::Describe, serde::Serialize, serde::Deserialize)]
        pub #com_strut

        impl #com_ident {
            #(#com_items)*
        }

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        pub struct #stub_ident {
            component: scrypto::constructs::Component,
        }

        impl #stub_ident {
            // need to reserve kw `from_address`
            pub fn from_address(address: scrypto::types::Address) -> Self {
                Self { component: address.into() }
            }

            #(#stub_items)*
        }

        #[no_mangle]
        pub extern "C" fn #dispatcher_ident(input_ptr: *mut u8) -> *mut u8 {
            // copy and release
            let input_bytes = scrypto::kernel::radix_copy(input_ptr);
            scrypto::kernel::radix_free(input_ptr);

            // deserialize the input
            let input = scrypto::buffer::radix_decode::<scrypto::abi::CallInput>(
                &input_bytes,
            );

            // invoke the method
            let rtn;
            match input.method.as_str() {
                #( #arm_guards => #arm_bodies, )*
                _ => {
                    panic!("Method not found: name = {}", input.method)
                }
            }

            // serialize the output
            let output = scrypto::abi::CallOutput { rtn };
            let output_bytes = scrypto::buffer::bincode_encode(&output);

            // return the output wrapped in a radix-style buffer
            unsafe {
                let ptr = scrypto::kernel::radix_alloc(output_bytes.len());
                std::ptr::copy(output_bytes.as_ptr(), ptr, output_bytes.len());
                ptr
            }
        }

        #[no_mangle]
        pub extern "C" fn #abi_ident() -> *mut u8 {
            extern crate alloc;
            use alloc::string::ToString;
            use scrypto::abi::{self, Describe};

            let output = abi::Component {
                name: #com_name.to_string(),
                methods: vec![
                    #(#abi_methods),*
                ],
            };

            // serialize the output
            let output_bytes = scrypto::buffer::radix_encode(&output);

            // return the output wrapped in a radix-style buffer
            unsafe {
                let ptr = scrypto::kernel::radix_alloc(output_bytes.len());
                std::ptr::copy(output_bytes.as_ptr(), ptr, output_bytes.len());
                ptr
            }
        }
    };
    trace!("handle_component() finishes");

    print_compiled_code("component!", &output);
    output.into()
}

// Parses function items in an `Impl` and returns the arm guards and bodies
// used for invocation matching.
fn generate_dispatcher(com_ident: &Ident, items: &Vec<ImplItem>) -> (Vec<Expr>, Vec<Expr>) {
    let mut arm_guards = Vec::<Expr>::new();
    let mut arm_bodies = Vec::<Expr>::new();

    for item in items {
        trace!("Processing function: {}", quote! { #item });

        match item {
            ImplItem::Method(ref m) => match &m.vis {
                Visibility::Public(_) => {
                    let fn_name = &m.sig.ident.to_string();
                    let fn_ident = &m.sig.ident;

                    trace!("[1] Generating argument loading statements...");
                    let mut args: Vec<Expr> = vec![];
                    let mut stmts: Vec<Stmt> = vec![];
                    let mut get_state: Option<Stmt> = None;
                    let mut put_state: Option<Stmt> = None;
                    let mut i: usize = 0;
                    for input in &m.sig.inputs {
                        let arg = format_ident!("arg{}", i);
                        match input {
                            FnArg::Receiver(ref r) => {
                                // Check receiver type and mutability
                                if r.reference.is_none() {
                                    panic!("Function input `self` is not supported. Consider replacing it with &self.");
                                }
                                let mutability = r.mutability;

                                // Generate an `Arg` and a loading `Stmt` for the i-th argument
                                let stmt: Stmt = parse_quote! {
                                    let #arg = scrypto::buffer::radix_decode::<scrypto::constructs::Component>(&input.args[#i]);
                                };
                                trace!("Stmt: {}", quote! { #stmt });
                                args.push(parse_quote! { & #mutability state });
                                stmts.push(stmt);

                                // Generate a `Stmt` for loading the component state
                                assert!(get_state.is_none(), "Can have at most 1 self reference");
                                get_state = Some(parse_quote! {
                                    let #mutability state: #com_ident = #arg.get_state();
                                });

                                // Generate a `Stmt` for writing back component state
                                if mutability.is_some() {
                                    put_state = Some(parse_quote! {
                                        #arg.put_state(&state);
                                    });
                                }
                            }
                            FnArg::Typed(ref t) => {
                                // Generate an `Arg` and a loading `Stmt` for the i-th argument
                                let ty = &t.ty;
                                let stmt: Stmt = parse_quote! {
                                    let #arg = scrypto::buffer::radix_decode::<#ty>(&input.args[#i]);
                                };
                                trace!("Stmt: {}", quote! { #stmt });
                                args.push(parse_quote! { #arg });
                                stmts.push(stmt);
                            }
                        }
                        i += 1;
                    }

                    trace!("[2] Generating function call statement...");
                    // load state if needed
                    if get_state.is_some() {
                        trace!("Stmt: {}", quote! { #get_state });
                        stmts.push(get_state.unwrap());
                    }
                    // invoke the function
                    let stmt: Stmt = parse_quote! {
                        rtn = scrypto::buffer::radix_encode(
                            &#com_ident::#fn_ident(#(#args),*)
                        );
                    };
                    trace!("Stmt: {}", quote! { #stmt });
                    stmts.push(stmt);
                    // update state
                    if put_state.is_some() {
                        trace!("Stmt: {}", quote! { #put_state });
                        stmts.push(put_state.unwrap());
                    }

                    trace!("[3] Generating match arm...");
                    arm_guards.push(parse_quote! { #fn_name });
                    arm_bodies.push(Expr::Block(ExprBlock {
                        attrs: vec![],
                        label: None,
                        block: Block {
                            brace_token: Brace {
                                span: Span::call_site(),
                            },
                            stmts,
                        },
                    }));
                }
                _ => {}
            },
            _ => {}
        };
    }

    (arm_guards, arm_bodies)
}

// Parses function items of an `Impl` and returns ABI of functions.
fn generate_abi(comp_name: &str, items: &Vec<ImplItem>) -> Vec<Expr> {
    let mut functions = Vec::<Expr>::new();

    for item in items {
        trace!("Processing function: {}", quote! { #item });
        match item {
            ImplItem::Method(ref m) => match &m.vis {
                Visibility::Public(_) => {
                    let name = m.sig.ident.to_string();
                    let mut kind = quote! { scrypto::abi::MethodKind::Functional };
                    let mut mutability = quote! { scrypto::abi::Mutability::Immutable };
                    let mut inputs = vec![];
                    for input in &m.sig.inputs {
                        match input {
                            FnArg::Receiver(ref r) => {
                                // Check receiver type and mutability
                                if r.reference.is_none() {
                                    panic!("Function input `self` is not supported. Consider replacing it with &self.");
                                }
                                kind = quote! { scrypto::abi::MethodKind::Stateful };

                                if r.mutability.is_some() {
                                    mutability = quote! { scrypto::abi::Mutability::Mutable };
                                }
                            }
                            FnArg::Typed(ref t) => {
                                let ty = replace_self_with(&t.ty, comp_name);
                                inputs.push(quote! {
                                    #ty::describe()
                                });
                            }
                        }
                    }

                    let output = match &m.sig.output {
                        ReturnType::Default => quote! {
                            scrypto::abi::Type::Unit
                        },
                        ReturnType::Type(_, t) => {
                            let ty = replace_self_with(t, comp_name);
                            quote! {
                                #ty::describe()
                            }
                        }
                    };

                    functions.push(parse_quote! {
                        scrypto::abi::Method {
                            name: #name.to_string(),
                            kind: #kind,
                            mutability: #mutability,
                            inputs: vec![#(#inputs),*],
                            output: #output,
                        }
                    });
                }
                _ => {}
            },
            _ => {
                panic!("Non-method impl items are not supported!")
            }
        };
    }

    functions
}

// Parses function items in an `Impl` and generates stubs for compile-time check.
fn generate_stub(com_ident: &Ident, items: &Vec<ImplItem>) -> Vec<Item> {
    let mut stubs: Vec<Item> = vec![];

    for item in items {
        trace!("Processing function: {}", quote! { #item });
        match item {
            ImplItem::Method(ref m) => match &m.vis {
                Visibility::Public(_) => {
                    // Check if this is a static method
                    let mut is_static = true;
                    for input in &m.sig.inputs {
                        match input {
                            FnArg::Receiver(ref r) => {
                                // Check receiver type and mutability
                                if r.reference.is_none() {
                                    panic!("Function input `self` is not supported. Consider replacing it with &self.");
                                }
                                is_static = false;
                            }
                            _ => {}
                        }
                    }
                    trace!("Static: {}", is_static);

                    let com_name = com_ident.to_string();
                    let method_ident = &m.sig.ident;
                    let method_name = method_ident.to_string();
                    let method_inputs = &m.sig.inputs;
                    let rtn_type = &m.sig.output;

                    // Generate argument list
                    let blueprint_arg: Option<Punctuated<FnArg, Comma>> = match is_static {
                        true => Some(parse_quote! { blueprint: &scrypto::constructs::Blueprint, }),
                        false => None,
                    };
                    let mut other_args = Punctuated::<Pat, Comma>::new();
                    for a in &m.sig.inputs {
                        match a {
                            FnArg::Receiver(_) => {}
                            FnArg::Typed(p) => {
                                other_args.push(*p.pat.clone());
                                other_args.push_punct(Comma(Span::call_site()));
                            }
                        }
                    }
                    trace!("Args: {}", quote! { #blueprint_arg, #other_args });

                    // Generate blueprint/component call
                    let stub: Item = match is_static {
                        true => match rtn_type {
                            ReturnType::Default => parse_quote! {
                                #[allow(dead_code)]
                                pub fn #method_ident(#blueprint_arg #method_inputs) {
                                    scrypto::call_blueprint!((), blueprint, #com_name, #method_name, #other_args)
                                }
                            },
                            ReturnType::Type(_, t) => parse_quote! {
                                #[allow(dead_code)]
                                pub fn #method_ident(#blueprint_arg #method_inputs) -> #t {
                                    scrypto::call_blueprint!(#t, blueprint, #com_name, #method_name, #other_args)
                                }
                            },
                        },
                        false => match rtn_type {
                            ReturnType::Default => parse_quote! {
                                #[allow(dead_code)]
                                pub fn #method_ident(#method_inputs) {
                                    scrypto::call_component!((), self.component, #method_name, #other_args)
                                }
                            },
                            ReturnType::Type(_, t) => parse_quote! {
                                #[allow(dead_code)]
                                pub fn #method_ident(#method_inputs) -> #t {
                                    scrypto::call_component!(#t, self.component, #method_name, #other_args)
                                }
                            },
                        },
                    };
                    trace!("Stub: {}", quote! { #stub });

                    stubs.push(stub);
                }
                _ => {}
            },
            _ => {}
        };
    }

    stubs
}

fn replace_self_with(t: &Type, name: &str) -> Type {
    match t {
        Type::Path(tp) => {
            let mut tp2 = tp.clone();
            tp2.path.segments.iter_mut().for_each(|s| {
                if s.ident.to_string() == "Self" {
                    s.ident = format_ident!("{}", name)
                }
            });
            Type::Path(tp2)
        }
        _ => t.clone(),
    }
}
