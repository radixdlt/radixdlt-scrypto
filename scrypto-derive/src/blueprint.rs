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

pub fn handle_blueprint(input: TokenStream) -> TokenStream {
    trace!("Started processing blueprint macro");

    // parse blueprint struct and impl
    let bp = match parse2::<ast::Blueprint>(input) {
        Ok(bp) => bp,
        Err(e) => {
            return e.to_compile_error();
        }
    };

    let bp_strut = &bp.structure;
    let bp_fields = &bp_strut.fields;
    let bp_semi_token = &bp_strut.semi_token;
    let bp_impl = &bp.implementation;
    let bp_ident = &bp_strut.ident;
    let bp_items = &bp_impl.items;
    let bp_name = bp_ident.to_string();
    trace!("Blueprint name: {}", bp_name);

    let output_mod = quote! {
        mod blueprint {
            use super::*;

            #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
            pub struct #bp_ident #bp_fields #bp_semi_token

            impl #bp_ident {
                #(#bp_items)*
            }

            impl ::scrypto::core::State for #bp_ident {
                fn name() -> &'static str {
                    #bp_name
                }
                fn instantiate(self) -> ::scrypto::core::Component {
                    ::scrypto::core::Component::new(self)
                }
            }
        }
    };
    trace!("Generated mod: \n{}", quote! { #output_mod });

    let dispatcher_ident = format_ident!("{}_main", bp_ident);
    let (arm_guards, arm_bodies) = generate_dispatcher(bp_ident, bp_items);
    let output_dispatcher = quote! {
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
    trace!("Generated dispatcher: \n{}", quote! { #output_dispatcher });

    let abi_ident = format_ident!("{}_abi", bp_ident);
    let (abi_functions, abi_methods) = generate_abi(bp_ident, bp_items);
    let output_abi = quote! {
        #[no_mangle]
        pub extern "C" fn #abi_ident() -> *mut u8 {
            use ::sbor::Describe;
            use ::scrypto::abi::{Function, Method};
            use ::scrypto::rust::borrow::ToOwned;
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
    trace!(
        "Generated ABI exporter: \n{}",
        quote! { #output_dispatcher }
    );

    let output_stubs = generate_stubs(bp_ident, bp_items);

    let output = quote! {
        #output_mod

        #output_dispatcher

        #output_abi

        #output_stubs
    };
    trace!("Finished processing blueprint macro");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("blueprint!", &output);

    output
}

// Parses function items in an `Impl` and returns the arm guards and bodies
// used for call matching.
fn generate_dispatcher(bp_ident: &Ident, items: &[ImplItem]) -> (Vec<Expr>, Vec<Expr>) {
    let mut arm_guards = Vec::<Expr>::new();
    let mut arm_bodies = Vec::<Expr>::new();

    for item in items {
        trace!("Processing function: {}", quote! { #item });

        if let ImplItem::Method(ref m) = item {
            if let Visibility::Public(_) = &m.vis {
                let fn_name = &m.sig.ident.to_string();
                let fn_ident = &m.sig.ident;

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
                                panic!("Function input `self` is not supported. Try replacing it with `&self`.");
                            }
                            let mutability = r.mutability;

                            // Generate an `Arg` and a loading `Stmt` for the i-th argument
                            let stmt: Stmt = parse_quote! {
                                let #arg = ::scrypto::core::Component::from(
                                    ::scrypto::utils::unwrap_light(
                                        ::scrypto::buffer::scrypto_decode::<::scrypto::types::Address>(&calldata.args[#i])
                                    )
                                );
                            };
                            trace!("Generated stmt: {}", quote! { #stmt });
                            args.push(parse_quote! { & #mutability state });
                            stmts.push(stmt);

                            // Generate a `Stmt` for loading the component state
                            assert!(get_state.is_none(), "Can have at most 1 self reference");
                            get_state = Some(parse_quote! {
                                let #mutability state: blueprint::#bp_ident = #arg.get_state();
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
                            trace!("Generated stmt: {}", quote! { #stmt });
                            args.push(parse_quote! { #arg });
                            stmts.push(stmt);
                        }
                    }
                }

                // load state if needed
                if let Some(stmt) = get_state {
                    trace!("Generated stmt: {}", quote! { #stmt });
                    stmts.push(stmt);
                }
                // call the function
                let stmt: Stmt = parse_quote! {
                    rtn = ::scrypto::buffer::scrypto_encode_for_kernel(
                        &blueprint::#bp_ident::#fn_ident(#(#args),*)
                    );
                };
                trace!("Generated stmt: {}", quote! { #stmt });
                stmts.push(stmt);
                // update state
                if let Some(stmt) = put_state {
                    trace!("Generated stmt: {}", quote! { #stmt });
                    stmts.push(stmt);
                }

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
fn generate_abi(bp_ident: &Ident, items: &[ImplItem]) -> (Vec<Expr>, Vec<Expr>) {
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
                                    panic!("Function input `self` is not supported. Try replacing it with &self.");
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
                                let ty = replace_self_with(&t.ty, &bp_ident.to_string());
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
                            let ty = replace_self_with(t, &bp_ident.to_string());
                            quote! {
                                <#ty>::describe()
                            }
                        }
                    };

                    if mutability.is_none() {
                        functions.push(parse_quote! {
                            ::scrypto::abi::Function {
                                name: #name.to_owned(),
                                inputs: vec![#(#inputs),*],
                                output: #output,
                            }
                        });
                    } else {
                        methods.push(parse_quote! {
                            ::scrypto::abi::Method {
                                name: #name.to_owned(),
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

// Parses function items of an `Impl` and returns ABI of functions.
fn generate_stubs(bp_ident: &Ident, items: &[ImplItem]) -> TokenStream {
    let bp_name = bp_ident.to_string();
    let mut functions = Vec::<ImplItem>::new();
    let mut methods = Vec::<ImplItem>::new();

    for item in items {
        trace!("Processing item: {}", quote! { #item });
        match item {
            ImplItem::Method(ref m) => {
                if let Visibility::Public(_) = &m.vis {
                    let ident = &m.sig.ident;
                    let name = ident.to_string();
                    let mut mutable = None;
                    let mut input_types = vec![];
                    let mut input_args = vec![];
                    let mut input_len = 0;
                    for input in &m.sig.inputs {
                        match input {
                            FnArg::Receiver(ref r) => {
                                // Check receiver type and mutability
                                if r.reference.is_none() {
                                    panic!("Function input `self` is not supported. Try replacing it with &self.");
                                }

                                if r.mutability.is_some() {
                                    mutable = Some(true);
                                } else {
                                    mutable = Some(false);
                                }
                            }
                            FnArg::Typed(ref t) => {
                                input_len += 1;
                                let arg = format_ident!("arg{}", input_len.to_string());
                                input_args.push(arg);

                                let ty = replace_self_with(&t.ty, &bp_ident.to_string());
                                input_types.push(ty);
                            }
                        }
                    }

                    let output = match &m.sig.output {
                        ReturnType::Default => parse_quote! { () },
                        ReturnType::Type(_, t) => replace_self_with(t, &bp_ident.to_string()),
                    };

                    if mutable.is_none() {
                        functions.push(parse_quote! {
                            pub fn #ident(#(#input_args: #input_types),*) -> #output {
                                let package = ::scrypto::core::Context::package_address();
                                let rtn = ::scrypto::core::call_function(
                                    package,
                                    #bp_name,
                                    #name,
                                    ::scrypto::args!(#(#input_args),*)
                                );
                                ::scrypto::utils::unwrap_light(::scrypto::buffer::scrypto_decode(&rtn))
                            }
                        });
                    } else {
                        methods.push(parse_quote! {
                            pub fn #ident(&self #(, #input_args: #input_types)*) -> #output {
                                let rtn = ::scrypto::core::call_method(
                                    self.address,
                                    #name,
                                    ::scrypto::args!(#(#input_args),*)
                                );
                                ::scrypto::utils::unwrap_light(::scrypto::buffer::scrypto_decode(&rtn))
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

    quote! {
        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
        pub struct #bp_ident {
            address: ::scrypto::types::Address,
        }

        impl #bp_ident {
            #(#functions)*

            #(#methods)*
        }

        impl From<::scrypto::types::Address> for #bp_ident {
            fn from(address: ::scrypto::types::Address) -> Self {
                Self {
                    address
                }
            }
        }

        impl From<#bp_ident> for ::scrypto::types::Address {
            fn from(a: #bp_ident) -> ::scrypto::types::Address {
                a.address
            }
        }

        impl From<::scrypto::core::Component> for #bp_ident {
            fn from(component: ::scrypto::core::Component) -> Self {
                Self {
                    address: component.into()
                }
            }
        }

        impl From<#bp_ident> for ::scrypto::core::Component {
            fn from(a: #bp_ident) -> ::scrypto::core::Component {
                a.address.into()
            }
        }
    }
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
        let output = handle_blueprint(input);

        assert_code_eq(
            output,
            quote! {
                mod blueprint {
                    use super::*;

                    #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                    pub struct Test {
                        a: u32
                    }

                    impl Test {
                        pub fn x(&self) -> u32 {
                            self.a
                        }
                    }

                    impl ::scrypto::core::State for Test {
                        fn name() -> &'static str {
                            "Test"
                        }
                        fn instantiate(self) -> ::scrypto::core::Component {
                            ::scrypto::core::Component::new(self)
                        }
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
                            let arg0 = ::scrypto::core::Component::from(::scrypto::utils::unwrap_light(
                                ::scrypto::buffer::scrypto_decode::<::scrypto::types::Address>(
                                    &calldata.args[0usize]
                                )
                            ));
                            let state: blueprint::Test = arg0.get_state();
                            rtn = ::scrypto::buffer::scrypto_encode_for_kernel(&blueprint::Test::x(&state));
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
                    use ::scrypto::rust::borrow::ToOwned;
                    use ::scrypto::rust::vec;
                    use ::scrypto::rust::vec::Vec;
                    let functions: Vec<Function> = vec![];
                    let methods: Vec<Method> = vec![::scrypto::abi::Method {
                        name: "x".to_owned(),
                        mutability: ::scrypto::abi::Mutability::Immutable,
                        inputs: vec![],
                        output: <u32>::describe(),
                    }];
                    let output = (functions, methods);
                    let output_bytes = ::scrypto::buffer::scrypto_encode_for_kernel(&output);
                    ::scrypto::buffer::scrypto_wrap(output_bytes)
                }
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                pub struct Test {
                    address: ::scrypto::types::Address,
                }
                impl Test {
                    pub fn x(&self) -> u32 {
                        let rtn = ::scrypto::core::call_method(self.address, "x", ::scrypto::args!());
                        ::scrypto::utils::unwrap_light(::scrypto::buffer::scrypto_decode(&rtn))
                    }
                }
                impl From<::scrypto::types::Address> for Test {
                    fn from(address: ::scrypto::types::Address) -> Self {
                        Self { address }
                    }
                }
                impl From<Test> for ::scrypto::types::Address {
                    fn from(a: Test) -> ::scrypto::types::Address {
                        a.address
                    }
                }
                impl From<::scrypto::core::Component> for Test {
                    fn from(component: ::scrypto::core::Component) -> Self {
                        Self {
                            address: component.into()
                        }
                    }
                }
                impl From<Test> for ::scrypto::core::Component {
                    fn from(a: Test) -> ::scrypto::core::Component {
                        a.address.into()
                    }
                }
            },
        );
    }
}
