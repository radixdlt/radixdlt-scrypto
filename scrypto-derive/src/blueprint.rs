use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::*;

use crate::ast;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_blueprint(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_blueprint() starts");

    // parse blueprint struct and impl
    let blueprint = parse2::<ast::Blueprint>(input)?;
    let bp = blueprint.module;
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

    let module_ident = bp.module_ident;
    let component_ident = format_ident!("{}Component", bp_ident);
    let component_ref_ident = format_ident!("{}GlobalComponentRef", bp_ident);
    let use_statements = {
        let mut use_statements = bp.use_statements;

        let contains_prelude_import = use_statements
            .iter()
            .map(|x| quote! { #(#x) }.to_string())
            .any(|x| x.contains("scrypto :: prelude :: *"));
        if !contains_prelude_import {
            let item: ItemUse = parse_quote! { use scrypto::prelude::*; };
            use_statements.push(item);
        }

        // TODO: remove
        let contains_super_import = use_statements
            .iter()
            .map(|x| quote! { #(#x) }.to_string())
            .any(|x| x.contains("super :: *"));
        if !contains_super_import {
            let item: ItemUse = parse_quote! { use super::*; };
            use_statements.push(item);
        }

        use_statements
    };

    let output_original_code = quote! {
        #[derive(::scrypto::prelude::ScryptoSbor)]
        pub struct #bp_ident #bp_fields #bp_semi_token

        impl #bp_ident {
            #(#bp_items)*
        }

        impl ::scrypto::component::ComponentState<#component_ident> for #bp_ident {
            fn instantiate(self) -> #component_ident {
                let component = ::scrypto::component::create_component(
                    #bp_name,
                    self
                );
                #component_ident {
                    component
                }
            }
        }
    };
    trace!("Generated mod: \n{}", quote! { #output_original_code });
    let method_input_structs = generate_method_input_structs(bp_ident, bp_items);

    let functions = generate_dispatcher(bp_ident, bp_items)?;
    let output_dispatcher = quote! {
        #(#method_input_structs)*
        #(#functions)*
    };

    trace!("Generated dispatcher: \n{}", quote! { #output_dispatcher });

    #[cfg(feature = "no-schema")]
    let output_schema = quote! {};
    #[cfg(not(feature = "no-schema"))]
    let output_schema = {
        let schema_ident = format_ident!("{}_schema", bp_ident);
        let (function_names, function_schemas) = generate_schema(bp_ident, bp_items)?;

        // Getting the event types if the event attribute is defined for the type
        let (event_type_names, event_type_paths) = {
            let mut paths = std::collections::BTreeMap::<String, Path>::new();
            for attribute in blueprint.attributes {
                if attribute.path.is_ident("events") {
                    let events_inner = parse2::<ast::EventsInner>(attribute.tokens)?;
                    for path in events_inner.paths.iter() {
                        let ident_string = quote! { #path }
                            .to_string()
                            .split(':')
                            .last()
                            .unwrap()
                            .trim()
                            .to_owned();
                        if let Some(..) = paths.insert(ident_string, path.clone()) {
                            return Err(Error::new(
                                path.span(),
                                "An event with an identical name has already been registered",
                            ));
                        }
                    }
                }
            }
            (
                paths.keys().into_iter().cloned().collect::<Vec<_>>(),
                paths.values().into_iter().cloned().collect::<Vec<_>>(),
            )
        };

        quote! {
            #[no_mangle]
            pub extern "C" fn #schema_ident() -> ::scrypto::engine::wasm_api::Slice {
                use ::scrypto::schema::*;
                use ::sbor::rust::prelude::*;
                use ::sbor::schema::*;
                use ::sbor::*;

                let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

                // Aggregate substates
                let mut substates = Vec::new();
                let type_index = aggregator.add_child_type_and_descendents::<#bp_ident>();
                substates.push(type_index);

                // Aggregate functions
                let mut functions = BTreeMap::new();
                #(
                    functions.insert(#function_names.to_string(), #function_schemas);
                )*

                // Aggregate event schemas
                let mut event_schema = BTreeMap::new();
                #({
                    let local_type_index = aggregator.add_child_type_and_descendents::<#event_type_paths>();
                    event_schema.insert(#event_type_names.to_owned(), local_type_index);
                })*

                let return_data = BlueprintSchema {
                    parent: None,
                    schema: generate_full_schema(aggregator),
                    substates,
                    functions,
                    virtual_lazy_load_functions: BTreeMap::new(),
                    event_schema
                };

                return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
            }
        }
    };
    trace!(
        "Generated SCHEMA exporter: \n{}",
        quote! { #output_dispatcher }
    );

    let output_stubs = generate_stubs(&component_ident, &component_ref_ident, bp_ident, bp_items)?;

    let output = quote! {
        pub mod #module_ident {
            #(#use_statements)*

            #output_original_code

            #output_dispatcher

            #output_schema

            #output_stubs
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("blueprint", &output);

    trace!("handle_blueprint() finishes");
    Ok(output)
}

fn generate_method_input_structs(bp_ident: &Ident, items: &[ImplItem]) -> Vec<ItemStruct> {
    let mut method_input_structs = Vec::new();

    for item in items {
        if let ImplItem::Method(method) = item {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }

            let mut args = Vec::new();
            let mut index: usize = 0;
            for input in (&method.sig.inputs).into_iter() {
                match input {
                    FnArg::Receiver(_) => {}
                    FnArg::Typed(ref t) => {
                        let arg_ident = format_ident!("arg{}", index);
                        index += 1;
                        let arg_type = t.ty.as_ref();
                        let arg: Field = Field::parse_named
                            .parse2(quote! {
                                #arg_ident : #arg_type
                            })
                            .unwrap();
                        args.push(arg)
                    }
                }
            }

            let input_struct_ident = format_ident!("{}_{}_Input", bp_ident, method.sig.ident);

            let method_input_struct: ItemStruct = parse_quote! {
                #[allow(non_camel_case_types)]
                #[derive(::scrypto::prelude::ScryptoSbor)]
                pub struct #input_struct_ident {
                    #(#args),*
                }
            };
            method_input_structs.push(method_input_struct);
        }
    }
    method_input_structs
}

fn generate_dispatcher(bp_ident: &Ident, items: &[ImplItem]) -> Result<Vec<TokenStream>> {
    let mut functions = Vec::new();

    for item in items {
        trace!("Processing item: {}", quote! { #item });

        if let ImplItem::Method(ref m) = item {
            if let Visibility::Public(_) = &m.vis {
                let ident = &m.sig.ident;

                let mut match_args: Vec<Expr> = vec![];
                let mut dispatch_args: Vec<Expr> = vec![];
                let mut stmts: Vec<Stmt> = vec![];
                let mut get_state: Option<Stmt> = None;
                for (i, input) in (&m.sig.inputs).into_iter().enumerate() {
                    match input {
                        FnArg::Receiver(ref r) => {
                            // Check receiver type and mutability
                            if r.reference.is_none() {
                                return Err(Error::new(r.span(), "Function input `self` is not supported. Try replacing it with `&self`."));
                            }
                            let mutability = r.mutability;

                            // Generate a `Stmt` for loading the component state
                            assert!(get_state.is_none(), "Can't have more than 1 self reference");
                            if mutability.is_some() {
                                // Generate an `Arg` and a loading `Stmt` for the i-th argument
                                dispatch_args.push(parse_quote! { state.deref_mut() });
                                get_state = Some(parse_quote! {
                                    let mut state: DataRefMut<#bp_ident> = component_data.get_mut();
                                });
                            } else {
                                dispatch_args.push(parse_quote! { state.deref() });
                                get_state = Some(parse_quote! {
                                    let state: DataRef<#bp_ident> = component_data.get();
                                });
                            }
                        }
                        FnArg::Typed(_) => {
                            let arg_index = if get_state.is_some() { i - 1 } else { i };
                            let arg = format_ident!("arg{}", arg_index);

                            match_args.push(parse_quote! { #arg });
                            dispatch_args.push(parse_quote! { input.#arg });
                        }
                    }
                }

                // parse args
                let input_struct_ident = format_ident!("{}_{}_Input", bp_ident, ident);
                stmts.push(parse_quote! {
                    let input: #input_struct_ident = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(args)).unwrap();
                });

                let is_method = get_state.is_some();

                // load component state if needed
                if let Some(stmt) = get_state {
                    trace!("Generated stmt: {}", quote! { #stmt });
                    stmts.push(parse_quote! {
                        let component_id: ::scrypto::prelude::NodeId = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(component_id)).unwrap();
                    });
                    stmts.push(parse_quote! {
                        let mut component_data = ::scrypto::runtime::ComponentStatePointer::new(component_id);
                    });
                    stmts.push(stmt);
                }

                // call the function/method
                let stmt: Stmt = parse_quote! {
                    let return_data = #bp_ident::#ident(#(#dispatch_args),*);
                };
                trace!("Generated stmt: {}", quote! { #stmt });
                stmts.push(stmt);

                // return
                let stmt: Stmt = parse_quote! {
                    return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                };
                trace!("Generated stmt: {}", quote! { #stmt });
                stmts.push(stmt);

                let fn_ident = format_ident!("{}_{}", bp_ident, ident);
                let extern_function = {
                    if is_method {
                        quote! {
                            #[no_mangle]
                            pub extern "C" fn #fn_ident(component_id: ::scrypto::engine::wasm_api::Buffer, args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                                use ::sbor::rust::ops::{Deref, DerefMut};

                                // Set up panic hook
                                ::scrypto::set_up_panic_hook();

                                #(#stmts)*
                            }
                        }
                    } else {
                        quote! {
                            #[no_mangle]
                            pub extern "C" fn #fn_ident(args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                                use ::sbor::rust::ops::{Deref, DerefMut};

                                // Set up panic hook
                                ::scrypto::set_up_panic_hook();

                                #(#stmts)*
                            }
                        }
                    }
                };
                functions.push(extern_function);
            }
        };
    }

    Ok(functions)
}

fn generate_stubs(
    component_ident: &Ident,
    component_ref_ident: &Ident,
    bp_ident: &Ident,
    items: &[ImplItem],
) -> Result<TokenStream> {
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
                                    return Err(Error::new(r.span(), "Function input `self` is not supported. Try replacing it with &self."));
                                }

                                if r.mutability.is_some() {
                                    mutable = Some(true);
                                } else {
                                    mutable = Some(false);
                                }
                            }
                            FnArg::Typed(ref t) => {
                                let arg = format_ident!("arg{}", input_len.to_string());
                                input_args.push(arg);

                                let ty = replace_self_with(&t.ty, &bp_ident.to_string());
                                input_types.push(ty);

                                input_len += 1;
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
                                ::scrypto::runtime::Runtime::call_function(
                                    ::scrypto::runtime::Runtime::package_address(),
                                    #bp_name,
                                    #name,
                                    scrypto_args!(#(#input_args),*)
                                )
                            }
                        });
                    } else {
                        methods.push(parse_quote! {
                            pub fn #ident(&self #(, #input_args: #input_types)*) -> #output {
                                self.component.call(#name, scrypto_args!(
                                    #(
                                       #input_args
                                    ),*
                                ))
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

    let output = quote! {
        #[allow(non_camel_case_types)]
        #[derive(::scrypto::prelude::ScryptoSbor)]
        pub struct #component_ident {
            pub component: ::scrypto::component::OwnedComponent,
        }

        impl ::scrypto::component::Component for #component_ident {
            fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
                self.component.call(method, args)
            }
            fn package_address(&self) -> ::scrypto::prelude::PackageAddress {
                self.component.package_address()
            }
            fn blueprint_name(&self) -> String {
                self.component.blueprint_name()
            }
        }


        impl ::scrypto::component::LocalComponent for #component_ident {
            fn globalize_with_modules(
                self,
                access_rules: AccessRules,
                metadata: Metadata,
                royalty: Royalty,
            ) -> ComponentAddress {
                self.component.globalize_with_modules(access_rules, metadata, royalty)
            }
        }

        impl #component_ident {
            #(#functions)*

            #(#methods)*
        }

        #[allow(non_camel_case_types)]
        pub struct #component_ref_ident {
            pub component: ::scrypto::component::GlobalComponentRef,
        }

        impl From<ComponentAddress> for #component_ref_ident {
            fn from(address: ComponentAddress) -> Self {
                Self {
                    component: ::scrypto::component::GlobalComponentRef(address)
                }
            }
        }

        impl ::scrypto::component::Component for #component_ref_ident {
            fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
                self.component.call(method, args)
            }
            fn package_address(&self) -> ::scrypto::prelude::PackageAddress {
                self.component.package_address()
            }
            fn blueprint_name(&self) -> String {
                self.component.blueprint_name()
            }
        }

        impl #component_ref_ident {
            pub fn access_rules(&self) -> AttachedAccessRules {
                self.component.access_rules()
            }

            pub fn metadata(&self) -> AttachedMetadata {
                self.component.metadata()
            }

            pub fn royalty(&self) -> AttachedRoyalty {
                self.component.royalty()
            }

            #(#methods)*
        }
    };

    Ok(output)
}

#[allow(dead_code)]
fn generate_schema(bp_ident: &Ident, items: &[ImplItem]) -> Result<(Vec<String>, Vec<Expr>)> {
    let mut function_names = Vec::<String>::new();
    let mut function_schemas = Vec::<Expr>::new();

    for item in items {
        trace!("Processing item: {}", quote! { #item });
        match item {
            ImplItem::Method(ref m) => {
                if let Visibility::Public(_) = &m.vis {
                    let function_name = m.sig.ident.to_string();
                    let mut receiver = None;
                    for input in &m.sig.inputs {
                        match input {
                            FnArg::Receiver(ref r) => {
                                // Check receiver type and mutability
                                if r.reference.is_none() {
                                    return Err(Error::new(r.span(), "Function input `self` is not supported. Try replacing it with &self."));
                                }

                                if r.mutability.is_some() {
                                    receiver =
                                        Some(quote! { ::scrypto::schema::Receiver::SelfRefMut });
                                } else {
                                    receiver =
                                        Some(quote! { ::scrypto::schema::Receiver::SelfRef });
                                }
                            }
                            FnArg::Typed(_) => {}
                        }
                    }

                    let input_struct_ident = format_ident!("{}_{}_Input", bp_ident, m.sig.ident);
                    let output_type: Type = match &m.sig.output {
                        ReturnType::Default => parse_quote! {
                            ()
                        },
                        ReturnType::Type(_, t) => replace_self_with(t, &bp_ident.to_string()),
                    };
                    let export_name = format!("{}_{}", bp_ident, m.sig.ident);

                    if receiver.is_none() {
                        function_names.push(function_name);
                        function_schemas.push(parse_quote! {
                            ::scrypto::schema::FunctionSchema {
                                receiver: Option::None,
                                input: aggregator.add_child_type_and_descendents::<#input_struct_ident>(),
                                output: aggregator.add_child_type_and_descendents::<#output_type>(),
                                export_name: #export_name.to_string(),
                            }
                        });
                    } else {
                        function_names.push(function_name);
                        function_schemas.push(parse_quote! {
                            ::scrypto::schema::FunctionSchema {
                                receiver: Option::Some(#receiver),
                                input: aggregator.add_child_type_and_descendents::<#input_struct_ident>(),
                                output: aggregator.add_child_type_and_descendents::<#output_type>(),
                                export_name: #export_name.to_string(),
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

    Ok((function_names, function_schemas))
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
    use super::*;
    use proc_macro2::TokenStream;
    use std::str::FromStr;

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
            "mod test { struct Test {a: u32, admin: ResourceManager} impl Test { pub fn x(&self, i: u32) -> u32 { i + self.a } pub fn y(i: u32) -> u32 { i * 2 } } }",
        )
            .unwrap();
        let output = handle_blueprint(input).unwrap();

        assert_code_eq(
            output,
            quote! {

                pub mod test {
                    use scrypto::prelude::*;
                    use super::*;

                    #[derive(::scrypto::prelude::ScryptoSbor)]
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

                    impl ::scrypto::component::ComponentState<TestComponent> for Test {
                        fn instantiate(self) -> TestComponent {
                            let component = ::scrypto::component::create_component(
                                "Test",
                                self
                            );
                            TestComponent {
                                component
                            }
                        }
                    }

                    #[allow(non_camel_case_types)]
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct Test_x_Input { arg0 : u32 }

                    #[allow(non_camel_case_types)]
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct Test_y_Input { arg0 : u32 }

                    #[no_mangle]
                    pub extern "C" fn Test_x(component_id: ::scrypto::engine::wasm_api::Buffer, args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                        use ::sbor::rust::ops::{Deref, DerefMut};

                        // Set up panic hook
                        ::scrypto::set_up_panic_hook();

                        let input: Test_x_Input = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(args)).unwrap();
                        let component_id: ::scrypto::prelude::NodeId = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(component_id)).unwrap();
                        let mut component_data = ::scrypto::runtime::ComponentStatePointer::new(component_id);
                        let state: DataRef<Test> = component_data.get();
                        let return_data = Test::x(state.deref(), input.arg0);
                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    #[no_mangle]
                    pub extern "C" fn Test_y(args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                        use ::sbor::rust::ops::{Deref, DerefMut};

                        // Set up panic hook
                        ::scrypto::set_up_panic_hook();

                        let input: Test_y_Input = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(args)).unwrap();
                        let return_data = Test::y(input.arg0);
                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    #[no_mangle]
                    pub extern "C" fn Test_schema() -> ::scrypto::engine::wasm_api::Slice {
                        use ::scrypto::schema::*;
                        use ::sbor::rust::prelude::*;
                        use ::sbor::schema::*;
                        use ::sbor::*;
                        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
                        let mut substates = Vec::new();
                        let type_index = aggregator.add_child_type_and_descendents::<Test>();
                        substates.push(type_index);
                        let mut functions = BTreeMap::new();
                        functions.insert(
                            "x".to_string(),
                            ::scrypto::schema::FunctionSchema {
                                receiver: Option::Some(::scrypto::schema::Receiver::SelfRef),
                                input: aggregator.add_child_type_and_descendents::<Test_x_Input>(),
                                output: aggregator.add_child_type_and_descendents::<u32>(),
                                export_name: "Test_x".to_string(),
                            }
                        );
                        functions.insert(
                            "y".to_string(),
                            ::scrypto::schema::FunctionSchema {
                                receiver: Option::None,
                                input: aggregator.add_child_type_and_descendents::<Test_y_Input>(),
                                output: aggregator.add_child_type_and_descendents::<u32>(),
                                export_name: "Test_y".to_string(),
                            }
                        );
                        let mut event_schema = BTreeMap::new();
                        let return_data = BlueprintSchema {
                            parent: None,
                            schema: generate_full_schema(aggregator),
                            substates,
                            functions,
                            virtual_lazy_load_functions: BTreeMap::new(),
                            event_schema
                        };
                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    #[allow(non_camel_case_types)]
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct TestComponent {
                        pub component: ::scrypto::component::OwnedComponent,
                    }

                    impl ::scrypto::component::Component for TestComponent {
                        fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
                            self.component.call(method, args)
                        }
                        fn package_address(&self) -> ::scrypto::prelude::PackageAddress {
                            self.component.package_address()
                        }
                        fn blueprint_name(&self) -> String {
                            self.component.blueprint_name()
                        }
                    }

                    impl ::scrypto::component::LocalComponent for TestComponent {
                        fn globalize_with_modules(
                            self,
                            access_rules: AccessRules,
                            metadata: Metadata,
                            royalty: Royalty,
                        ) -> ComponentAddress {
                            self.component.globalize_with_modules(access_rules, metadata, royalty)
                        }
                    }

                    impl TestComponent {
                        pub fn y(arg0: u32) -> u32 {
                            ::scrypto::runtime::Runtime::call_function(::scrypto::runtime::Runtime::package_address(), "Test", "y", scrypto_args!(arg0))
                        }

                        pub fn x(&self, arg0: u32) -> u32 {
                            self.component.call("x", scrypto_args!(arg0))
                        }
                    }

                    #[allow(non_camel_case_types)]
                    pub struct TestGlobalComponentRef {
                        pub component: ::scrypto::component::GlobalComponentRef,
                    }

                    impl From<ComponentAddress> for TestGlobalComponentRef {
                        fn from(address: ComponentAddress) -> Self {
                            Self {
                                component: ::scrypto::component::GlobalComponentRef(address)
                            }
                        }
                    }

                    impl ::scrypto::component::Component for TestGlobalComponentRef {
                        fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
                            self.component.call(method, args)
                        }
                        fn package_address(&self) -> ::scrypto::prelude::PackageAddress {
                            self.component.package_address()
                        }
                        fn blueprint_name(&self) -> String {
                            self.component.blueprint_name()
                        }
                    }

                    impl TestGlobalComponentRef {
                        pub fn access_rules(&self) -> AttachedAccessRules {
                            self.component.access_rules()
                        }

                        pub fn metadata(&self) -> AttachedMetadata {
                            self.component.metadata()
                        }

                        pub fn royalty(&self) -> AttachedRoyalty {
                            self.component.royalty()
                        }

                        pub fn x(&self, arg0: u32) -> u32 {
                            self.component.call("x", scrypto_args!(arg0))
                        }
                    }
                }
            },
        );
    }
}
