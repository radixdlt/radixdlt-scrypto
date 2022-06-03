use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::*;

use crate::ast;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_blueprint(input: TokenStream) -> Result<TokenStream> {
    trace!("Started processing blueprint macro");

    // parse blueprint struct and impl
    let bp = parse2::<ast::Blueprint>(input)?;
    let bp_strut = &bp.structure;
    let bp_fields = &bp_strut.fields;
    let bp_semi_token = &bp_strut.semi_token;
    let bp_impl = &bp.implementation;
    let bp_ident = &bp_strut.ident;
    let bp_items = &bp_impl.items;
    let bp_name = bp_ident.to_string();
    trace!("Blueprint name: {}", bp_name);

    let impl_ident_matches = match &*bp_impl.self_ty {
        Type::Path(p) => p
            .path
            .get_ident()
            .filter(|i| i.to_string() == bp_name)
            .is_some(),
        _ => false,
    };
    if !impl_ident_matches {
        return Err(Error::new(
            bp_impl.span(),
            format!("Only `impl {}` is allowed here", bp_name),
        ));
    }

    let output_mod = quote! {
        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
        pub struct #bp_ident #bp_fields #bp_semi_token

        impl #bp_ident {
            #(#bp_items)*
        }

        impl ::scrypto::component::ComponentState for #bp_ident {
            fn instantiate(self) -> ::scrypto::component::LocalComponent {
                ::scrypto::component::component_system().to_component_state_with_auth(
                    #bp_name,
                    self
                )
            }
        }
    };
    trace!("Generated mod: \n{}", quote! { #output_mod });
    let method_enum_ident = format_ident!("{}Method", bp_ident);
    let method_enum = generate_method_enum(&method_enum_ident, bp_items);

    let dispatcher_ident = format_ident!("{}_main", bp_ident);
    let (arm_guards, arm_bodies) = generate_dispatcher(&method_enum_ident, bp_ident, bp_items)?;
    let output_dispatcher = if arm_guards.is_empty() {
        quote! {
            #method_enum

            #[no_mangle]
            pub extern "C" fn #dispatcher_ident(input: *mut u8) -> *mut u8 {
                panic!("No invocation expected")
            }
        }
    } else {
        quote! {
            #method_enum

            #[no_mangle]
            pub extern "C" fn #dispatcher_ident(input: *mut u8) -> *mut u8 {
                // Set up panic hook
                ::scrypto::misc::set_up_panic_hook();

                // Set up component and resource subsystems;
                ::scrypto::component::init_component_system(::scrypto::component::ComponentSystem::new());
                ::scrypto::resource::init_resource_system(::scrypto::resource::ResourceSystem::new());

                // Dispatch the call
                let method = ::scrypto::buffer::scrypto_decode_from_buffer::<#method_enum_ident>(input).unwrap();
                let rtn;
                match method {
                    #( #arm_guards => #arm_bodies )*
                }

                // Return
                rtn
            }
        }
    };

    trace!("Generated dispatcher: \n{}", quote! { #output_dispatcher });

    let abi_ident = format_ident!("{}_abi", bp_ident);
    let (abi_functions, abi_methods) = generate_abi(bp_ident, bp_items)?;
    let output_abi = quote! {
        #[no_mangle]
        pub extern "C" fn #abi_ident(input: *mut u8) -> *mut u8 {
            use ::sbor::{Describe, Type};
            use ::scrypto::abi::{Function, Method};
            use ::sbor::rust::borrow::ToOwned;
            use ::sbor::rust::vec;
            use ::sbor::rust::vec::Vec;

            let functions: Vec<Function> = vec![ #(#abi_functions),* ];
            let methods: Vec<Method> = vec![ #(#abi_methods),* ];
            let schema: Type = #bp_ident::describe();
            let output = (schema, functions, methods);

            ::scrypto::buffer::scrypto_encode_to_buffer(&output)
        }
    };
    trace!(
        "Generated ABI exporter: \n{}",
        quote! { #output_dispatcher }
    );

    let output = quote! {
        #output_mod

        #output_dispatcher

        #output_abi
    };
    trace!("Finished processing blueprint macro");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("blueprint!", &output);

    Ok(output)
}

fn generate_method_enum(method_enum_ident: &Ident, items: &[ImplItem]) -> ItemEnum {
    let mut variants = Vec::new();

    for item in items {
        if let ImplItem::Method(method) = item {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }

            let mut fields = Vec::new();
            for input in (&method.sig.inputs).into_iter() {
                match input {
                    FnArg::Receiver(_) => {}
                    FnArg::Typed(ref t) => {
                        fields.push(t.ty.as_ref());
                    }
                }
            }

            let method_ident = method.sig.ident.clone();
            let variant: Variant = parse_quote! {
                #method_ident(#(#fields),*)
            };
            variants.push(variant);
        }
    }

    parse_quote! {
        #[allow(non_camel_case_types)]
        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
        enum #method_enum_ident {
            #(#variants),*
        }
    }
}

// Parses function items in an `Impl` and returns the arm guards and bodies
// used for call matching.
fn generate_dispatcher(
    method_enum_ident: &Ident,
    bp_ident: &Ident,
    items: &[ImplItem],
) -> Result<(Vec<Expr>, Vec<Expr>)> {
    let mut arm_guards = Vec::<Expr>::new();
    let mut arm_bodies = Vec::<Expr>::new();

    for item in items {
        trace!("Processing item: {}", quote! { #item });

        if let ImplItem::Method(ref m) = item {
            if let Visibility::Public(_) = &m.vis {
                let ident = &m.sig.ident;

                let mut match_args: Vec<Expr> = vec![];
                let mut dispatch_args: Vec<Expr> = vec![];
                let mut stmts: Vec<Stmt> = vec![];
                let mut get_state: Option<Stmt> = None;
                let mut put_state: Option<Stmt> = None;
                for (i, input) in (&m.sig.inputs).into_iter().enumerate() {
                    match input {
                        FnArg::Receiver(ref r) => {
                            // Check receiver type and mutability
                            if r.reference.is_none() {
                                return Err(Error::new(r.span(), "Function input `self` is not supported. Try replacing it with `&self`."));
                            }
                            let mutability = r.mutability;

                            // Generate an `Arg` and a loading `Stmt` for the i-th argument
                            dispatch_args.push(parse_quote! { & #mutability state });

                            // Generate a `Stmt` for loading the component state
                            assert!(get_state.is_none(), "Can't have more than 1 self reference");
                            get_state = Some(parse_quote! {
                                let #mutability state: #bp_ident = borrow_component!(component_address).get_state();
                            });

                            // Generate a `Stmt` for writing back component state
                            if mutability.is_some() {
                                put_state = Some(parse_quote! {
                                    ::scrypto::borrow_component!(component_address).put_state(state);
                                });
                            }
                        }
                        FnArg::Typed(_) => {
                            let arg_index = if get_state.is_some() { i - 1 } else { i };
                            let arg = format_ident!("arg{}", arg_index);

                            match_args.push(parse_quote! { #arg });
                            dispatch_args.push(parse_quote! { #arg });
                        }
                    }
                }

                // load state if needed
                if let Some(stmt) = get_state {
                    trace!("Generated stmt: {}", quote! { #stmt });
                    stmts.push(parse_quote!{
                        let component_address = ::scrypto::core::Runtime::actor().component_address().unwrap();
                    });
                    stmts.push(stmt);
                }

                // call the function
                let stmt: Stmt = parse_quote! {
                    rtn = ::scrypto::buffer::scrypto_encode_to_buffer(
                        &#bp_ident::#ident(#(#dispatch_args),*)
                    );
                };
                trace!("Generated stmt: {}", quote! { #stmt });
                stmts.push(stmt);

                // update state
                if let Some(stmt) = put_state {
                    trace!("Generated stmt: {}", quote! { #stmt });
                    stmts.push(stmt);
                }

                arm_guards.push(parse_quote! { #method_enum_ident::#ident(#(#match_args),*) });
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

    Ok((arm_guards, arm_bodies))
}

// Parses function items of an `Impl` and returns ABI of functions.
fn generate_abi(bp_ident: &Ident, items: &[ImplItem]) -> Result<(Vec<Expr>, Vec<Expr>)> {
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
                                    return Err(Error::new(r.span(), "Function input `self` is not supported. Try replacing it with &self."));
                                }

                                if r.mutability.is_some() {
                                    mutability =
                                        Some(quote! { ::scrypto::abi::SelfMutability::Mutable });
                                } else {
                                    mutability =
                                        Some(quote! { ::scrypto::abi::SelfMutability::Immutable });
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
                return Err(Error::new(
                    item.span(),
                    "Non-method impl items are not supported!",
                ));
            }
        };
    }

    Ok((functions, methods))
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
    #[should_panic]
    fn test_inconsistent_names_should_fail() {
        let input = TokenStream::from_str("struct A {} impl B { }").unwrap();
        handle_blueprint(input).unwrap();
    }

    #[test]
    fn test_blueprint() {
        let input = TokenStream::from_str(
            "struct Test {a: u32, admin: ResourceManager} impl Test { pub fn x(&self, i: u32) -> u32 { i + self.a } pub fn y(i: u32) -> u32 { i * 2 } }",
        )
        .unwrap();
        let output = handle_blueprint(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                pub struct Test {
                    a: u32,
                    admin: ResourceManager
                }

                impl Test {
                    pub fn x(&self, i: u32) -> u32 {
                        i + self.a
                    }
                    pub fn y(i: u32) -> u32 {
                        i * 2
                    }
                }

                impl ::scrypto::component::ComponentState for Test {
                    fn instantiate(self) -> ::scrypto::component::LocalComponent {
                        ::scrypto::component::component_system().to_component_state_with_auth(
                            "Test",
                            self
                        )
                    }
                }

                #[allow(non_camel_case_types)]
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                enum TestMethod {
                    x(u32),
                    y(u32)
                }

                #[no_mangle]
                pub extern "C" fn Test_main(input: *mut u8) -> *mut u8 {
                    ::scrypto::misc::set_up_panic_hook();
                    ::scrypto::component::init_component_system(::scrypto::component::ComponentSystem::new());
                    ::scrypto::resource::init_resource_system(::scrypto::resource::ResourceSystem::new());

                    let method = ::scrypto::buffer::scrypto_decode_from_buffer::<TestMethod>(input).unwrap();
                    let rtn;
                    match method {
                        TestMethod::x(arg0) => {
                            let component_address = ::scrypto::core::Runtime::actor().component_address().unwrap();
                            let state: Test = borrow_component!(component_address).get_state();
                            rtn = ::scrypto::buffer::scrypto_encode_to_buffer(&Test::x(&state, arg0));
                        }
                        TestMethod::y(arg0) => {
                            rtn = ::scrypto::buffer::scrypto_encode_to_buffer(&Test::y(arg0));
                        }
                    }
                    rtn
                }
                #[no_mangle]
                pub extern "C" fn Test_abi(input: *mut u8) -> *mut u8 {
                    use ::sbor::{Describe, Type};
                    use ::scrypto::abi::{Function, Method};
                    use ::sbor::rust::borrow::ToOwned;
                    use ::sbor::rust::vec;
                    use ::sbor::rust::vec::Vec;
                    let functions: Vec<Function> = vec![::scrypto::abi::Function {
                        name: "y".to_owned(),
                        inputs: vec![<u32>::describe()],
                        output: <u32>::describe(),
                    }];
                    let methods: Vec<Method> = vec![::scrypto::abi::Method {
                        name: "x".to_owned(),
                        mutability: ::scrypto::abi::SelfMutability::Immutable,
                        inputs: vec![<u32>::describe()],
                        output: <u32>::describe(),
                    }];
                    let schema: Type = Test::describe();
                    let output = (schema, functions, methods);
                    ::scrypto::buffer::scrypto_encode_to_buffer(&output)
                }
            },
        );
    }

    #[test]
    fn test_empty_blueprint() {
        let input = TokenStream::from_str("struct Test {} impl Test {}").unwrap();
        let output = handle_blueprint(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                pub struct Test {
                }

                impl Test {
                }

                impl ::scrypto::component::ComponentState for Test {
                    fn instantiate(self) -> ::scrypto::component::LocalComponent {
                        ::scrypto::component::component_system().to_component_state_with_auth(
                            "Test",
                            self
                        )
                    }
                }

                #[allow(non_camel_case_types)]
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                enum TestMethod {
                }

                #[no_mangle]
                pub extern "C" fn Test_main(input: *mut u8) -> *mut u8 {
                    panic!("No invocation expected")
                }
                #[no_mangle]
                pub extern "C" fn Test_abi(input: *mut u8) -> *mut u8 {
                    use ::sbor::{Describe, Type};
                    use ::scrypto::abi::{Function, Method};
                    use ::sbor::rust::borrow::ToOwned;
                    use ::sbor::rust::vec;
                    use ::sbor::rust::vec::Vec;
                    let functions: Vec<Function> = vec![];
                    let methods: Vec<Method> = vec![];
                    let schema: Type = Test::describe();
                    let output = (schema, functions, methods);
                    ::scrypto::buffer::scrypto_encode_to_buffer(&output)
                }
            },
        );
    }
}
