use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::token::Brace;
use syn::*;

use crate::ast;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_blueprint(input: TokenStream, output_abi: bool) -> TokenStream {
    trace!("handle_blueprint() begins");

    // parse blueprint struct and impl
    let result: Result<ast::Blueprint> = parse2(input);
    if result.is_err() {
        return result.err().unwrap().to_compile_error();
    }

    let bp = result.ok().unwrap();
    let bp_strut = &bp.structure;
    let bp_impl = &bp.implementation;
    let bp_ident = &bp_strut.ident;
    let bp_items = &bp_impl.items;
    let bp_name = bp_ident.to_string();

    trace!("Processing blueprint: {}", bp_name);
    let generated_blueprint = quote! {
        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
        pub #bp_strut

        impl #bp_ident {
            #(#bp_items)*
        }

        impl ::scrypto::traits::Blueprint for #bp_ident {
            fn name() -> &'static str {
                #bp_name
            }
        }

        impl #bp_ident {
            fn instantiate(self) -> ::scrypto::types::Address {
                ::scrypto::constructs::Component::new(self).address()
            }
        }
    };

    trace!("Generating dispatcher function...");
    let dispatcher_ident = format_ident!("{}_main", bp_ident);
    let (arm_guards, arm_bodies) = generate_dispatcher(bp_ident, bp_items);
    let generated_dispatcher = quote! {
        #[no_mangle]
        pub extern "C" fn #dispatcher_ident() -> *mut u8 {
            // Retrieve call data
            let calldata: ::scrypto::kernel::GetCallDataOutput = ::scrypto::kernel::call_kernel(
                ::scrypto::kernel::GET_CALL_DATA,
                ::scrypto::kernel::GetCallDataInput {},
            );

            // Dispatch the call
            let rtn;
            match calldata.function.as_str() {
                #( #arm_guards => #arm_bodies )*
                _ => {
                    panic!();
                }
            }

            // Return
            ::scrypto::buffer::scrypto_wrap(rtn)
        }
    };

    trace!("Generating ABI function...");
    let abi_ident = format_ident!("{}_abi", bp_ident);
    let (abi_functions, abi_methods) = generate_abi(&bp_name, bp_items);
    let generated_abi = quote! {
        #[no_mangle]
        pub extern "C" fn #abi_ident() -> *mut u8 {
            use ::sbor::Describe;
            use ::scrypto::abi::{Function, Method};
            use ::scrypto::rust::string::ToString;
            use ::scrypto::rust::vec;
            use ::scrypto::rust::vec::Vec;

            let functions: Vec<Function> = vec![ #(#abi_functions),* ];
            let methods: Vec<Method> = vec![ #(#abi_methods),* ];
            let output = (functions, methods);

            // serialize the output
            let output_bytes = ::scrypto::buffer::scrypto_encode_for_kernel(&output);

            // return the output wrapped in a radix-style buffer
            ::scrypto::buffer::scrypto_wrap(output_bytes)
        }
    };

    let optional_abi = output_abi.then(|| generated_abi);
    let output = quote! {
        #generated_blueprint

        #generated_dispatcher

        #optional_abi
    };
    trace!("handle_blueprint() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("blueprint!", &output);

    output
}

// Parses function items in an `Impl` and returns the arm guards and bodies
// used for invocation matching.
fn generate_dispatcher(bp_ident: &Ident, items: &[ImplItem]) -> (Vec<Expr>, Vec<Expr>) {
    let mut arm_guards = Vec::<Expr>::new();
    let mut arm_bodies = Vec::<Expr>::new();

    for item in items {
        trace!("Processing function: {}", quote! { #item });

        if let ImplItem::Method(ref m) = item {
            if let Visibility::Public(_) = &m.vis {
                let fn_name = &m.sig.ident.to_string();
                let fn_ident = &m.sig.ident;

                trace!("[1] Generating argument loading statements...");
                let mut args: Vec<Expr> = vec![];
                let mut stmts: Vec<Stmt> = vec![];
                let mut get_state: Option<Stmt> = None;
                let mut put_state: Option<Stmt> = None;
                for (i, input) in (&m.sig.inputs).into_iter().enumerate() {
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
                                let #arg = ::scrypto::constructs::Component::from(
                                    ::scrypto::utils::unwrap_light(
                                        ::scrypto::buffer::scrypto_decode::<::scrypto::types::Address>(&calldata.args[#i])
                                    )
                                );
                            };
                            trace!("Stmt: {}", quote! { #stmt });
                            args.push(parse_quote! { & #mutability state });
                            stmts.push(stmt);

                            // Generate a `Stmt` for loading the component state
                            assert!(get_state.is_none(), "Can have at most 1 self reference");
                            get_state = Some(parse_quote! {
                                let #mutability state: #bp_ident = #arg.get_state();
                            });

                            // Generate a `Stmt` for writing back component state
                            if mutability.is_some() {
                                put_state = Some(parse_quote! {
                                    #arg.put_state(state);
                                });
                            }
                        }
                        FnArg::Typed(ref t) => {
                            // Generate an `Arg` and a loading `Stmt` for the i-th argument
                            let ty = &t.ty;
                            let stmt: Stmt = parse_quote! {
                                let #arg = ::scrypto::utils::unwrap_light(
                                    ::scrypto::buffer::scrypto_decode::<#ty>(&calldata.args[#i])
                                );
                            };
                            trace!("Stmt: {}", quote! { #stmt });
                            args.push(parse_quote! { #arg });
                            stmts.push(stmt);
                        }
                    }
                }

                trace!("[2] Generating function call statement...");
                // load state if needed
                if let Some(s) = get_state {
                    trace!("Stmt: {}", quote! { #s });
                    stmts.push(s);
                }
                // call the function
                let stmt: Stmt = parse_quote! {
                    rtn = ::scrypto::buffer::scrypto_encode_for_kernel(
                        &#bp_ident::#fn_ident(#(#args),*)
                    );
                };
                trace!("Stmt: {}", quote! { #stmt });
                stmts.push(stmt);
                // update state
                if let Some(s) = put_state {
                    trace!("Stmt: {}", quote! { #s });
                    stmts.push(s);
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
        };
    }

    (arm_guards, arm_bodies)
}

// Parses function items of an `Impl` and returns ABI of functions.
fn generate_abi(bp_name: &str, items: &[ImplItem]) -> (Vec<Expr>, Vec<Expr>) {
    let mut functions = Vec::<Expr>::new();
    let mut methods = Vec::<Expr>::new();

    for item in items {
        trace!("Processing item: {}", quote! { #item });
        match item {
            ImplItem::Method(ref m) => {
                if let Visibility::Public(_) = &m.vis {
                    let name = m.sig.ident.to_string();
                    let mut mutability = None;
                    let mut inputs = vec![];
                    for input in &m.sig.inputs {
                        match input {
                            FnArg::Receiver(ref r) => {
                                // Check receiver type and mutability
                                if r.reference.is_none() {
                                    panic!("Function input `self` is not supported. Consider replacing it with &self.");
                                }

                                if r.mutability.is_some() {
                                    mutability =
                                        Some(quote! { ::scrypto::abi::Mutability::Mutable });
                                } else {
                                    mutability =
                                        Some(quote! { ::scrypto::abi::Mutability::Immutable });
                                }
                            }
                            FnArg::Typed(ref t) => {
                                let ty = replace_self_with(&t.ty, bp_name);
                                inputs.push(quote! {
                                    <#ty>::describe()
                                });
                            }
                        }
                    }

                    let output = match &m.sig.output {
                        ReturnType::Default => quote! {
                            ::sbor::describe::Type::Unit
                        },
                        ReturnType::Type(_, t) => {
                            let ty = replace_self_with(t, bp_name);
                            quote! {
                                <#ty>::describe()
                            }
                        }
                    };

                    if mutability.is_none() {
                        functions.push(parse_quote! {
                            ::scrypto::abi::Function {
                                name: #name.to_string(),
                                inputs: vec![#(#inputs),*],
                                output: #output,
                            }
                        });
                    } else {
                        methods.push(parse_quote! {
                            ::scrypto::abi::Method {
                                name: #name.to_string(),
                                mutability: #mutability,
                                inputs: vec![#(#inputs),*],
                                output: #output,
                            }
                        });
                    }
                }
            }
            _ => {
                panic!("Non-method impl items are not supported!")
            }
        };
    }

    (functions, methods)
}

fn replace_self_with(t: &Type, name: &str) -> Type {
    match t {
        Type::Path(tp) => {
            let mut tp2 = tp.clone();
            tp2.path.segments.iter_mut().for_each(|s| {
                if s.ident == "Self" {
                    s.ident = format_ident!("{}", name)
                }
            });
            Type::Path(tp2)
        }
        _ => t.clone(),
    }
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
    fn test_blueprint() {
        let input = TokenStream::from_str(
            "struct Test {a: u32} impl Test { pub fn x(&self) -> u32 { self.a } }",
        )
        .unwrap();
        let output = handle_blueprint(input, true);

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                pub struct Test {
                    a: u32
                }
                impl Test {
                    pub fn x(&self) -> u32 {
                        self.a
                    }
                }
                impl ::scrypto::traits::Blueprint for Test {
                    fn name() -> &'static str {
                        "Test"
                    }
                }
                impl Test {
                    fn instantiate(self) -> ::scrypto::types::Address {
                        ::scrypto::constructs::Component::new(self).address()
                    }
                }
                #[no_mangle]
                pub extern "C" fn Test_main() -> *mut u8 {
                    let calldata: ::scrypto::kernel::GetCallDataOutput = ::scrypto::kernel::call_kernel(
                        ::scrypto::kernel::GET_CALL_DATA,
                        ::scrypto::kernel::GetCallDataInput {},
                    );
                    let rtn;
                    match calldata.function.as_str() {
                        "x" => {
                            let arg0 = ::scrypto::constructs::Component::from(::scrypto::utils::unwrap_light(
                                ::scrypto::buffer::scrypto_decode::<::scrypto::types::Address>(
                                    &calldata.args[0usize]
                            )));
                            let state: Test = arg0.get_state();
                            rtn = ::scrypto::buffer::scrypto_encode_for_kernel(&Test::x(&state));
                        }
                        _ => {
                            panic!();
                        }
                    }
                    ::scrypto::buffer::scrypto_wrap(rtn)
                }
                #[no_mangle]
                pub extern "C" fn Test_abi() -> *mut u8 {
                    use ::sbor::Describe;
                    use ::scrypto::abi::{Function, Method};
                    use ::scrypto::rust::string::ToString;
                    use ::scrypto::rust::vec;
                    use ::scrypto::rust::vec::Vec;
                    let functions: Vec<Function> = vec![];
                    let methods: Vec<Method> = vec![::scrypto::abi::Method {
                        name: "x".to_string(),
                        mutability: ::scrypto::abi::Mutability::Immutable,
                        inputs: vec![],
                        output: <u32>::describe(),
                    }];
                    let output = (functions, methods);
                    let output_bytes = ::scrypto::buffer::scrypto_encode_for_kernel(&output);
                    ::scrypto::buffer::scrypto_wrap(output_bytes)
                }
            },
        );
    }
}
