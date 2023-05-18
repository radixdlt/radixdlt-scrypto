use proc_macro2::{Ident, Span, TokenStream};
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
    let mut bp = blueprint.module;
    let bp_strut = &bp.structure;
    let bp_fields = &bp_strut.fields;
    let bp_semi_token = &bp_strut.semi_token;
    let bp_impl = &mut bp.implementation;
    let bp_ident = &bp_strut.ident;
    validate_type_ident(&bp_ident)?;
    let bp_items = &mut bp_impl.items;
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
    let stub_ident = format_ident!("{}ObjectStub", bp_ident);
    validate_type_ident(&stub_ident)?;
    let functions_ident = format_ident!("{}Functions", bp_ident);
    validate_type_ident(&functions_ident)?;
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

    let generated_schema_info = generate_schema(bp_ident, bp_items)?;

    #[cfg(feature = "no-schema")]
    let output_schema = quote! {};
    let _ = generated_schema_info;
    #[cfg(not(feature = "no-schema"))]
    let output_schema = {
        let function_names = generated_schema_info.function_names;
        let function_schemas = generated_schema_info.function_schemas;
        let function_authority_names = generated_schema_info.function_authority_names;
        let function_mapped_authorities = generated_schema_info.function_mapped_authorities;

        let schema_ident = format_ident!("{}_schema", bp_ident);

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

                // Aggregate fields
                let mut fields = Vec::new();
                let type_index = aggregator.add_child_type_and_descendents::<#bp_ident>();
                fields.push(type_index);

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

                let mut authority_schema = BTreeMap::new();

                let mut method_authority_mapping = BTreeMap::new();
                #({
                    method_authority_mapping.insert(#function_authority_names.to_owned(), SchemaAuthorityKey::new(#function_mapped_authorities));
                })*

                let return_data = BlueprintSchema {
                    outer_blueprint: None,
                    schema: generate_full_schema(aggregator),
                    fields,
                    collections: Vec::new(),
                    functions,
                    virtual_lazy_load_functions: BTreeMap::new(),
                    event_schema,
                    method_authority_mapping,
                    authority_schema,
                };

                return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
            }
        }
    };

    let output_original_code = quote! {
        #[derive(::scrypto::prelude::ScryptoSbor)]
        pub struct #bp_ident #bp_fields #bp_semi_token

        impl #bp_ident {
            #(#bp_items)*
        }

        impl ::scrypto::component::ComponentState for #bp_ident {
            const BLUEPRINT_NAME: &'static str = #bp_name;
        }

        impl HasStub for #bp_ident {
            type Stub = #stub_ident;
        }
    };
    trace!("Generated mod: \n{}", quote! { #output_original_code });
    let method_input_structs = generate_method_input_structs(bp_ident, bp_items)?;

    let functions = generate_dispatcher(bp_ident, bp_items)?;
    let output_dispatcher = quote! {
        #(#method_input_structs)*
        #(#functions)*
    };

    trace!("Generated dispatcher: \n{}", quote! { #output_dispatcher });

    let output_stubs = generate_stubs(&stub_ident, &functions_ident, bp_ident, bp_items)?;

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

fn generate_method_input_structs(bp_ident: &Ident, items: &[ImplItem]) -> Result<Vec<ItemStruct>> {
    let mut method_input_structs = Vec::new();

    for item in items {
        if let ImplItem::Method(method) = item {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }

            let mut args = Vec::new();
            let mut index: usize = 0;
            for input in method.sig.inputs.iter() {
                match input {
                    FnArg::Receiver(_) => {}
                    FnArg::Typed(argument_and_type) => {
                        let arg_ident =
                            create_argument_ident(argument_and_type.pat.as_ref(), index)?;
                        index += 1;
                        let arg_type = argument_and_type.ty.as_ref();
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
            validate_type_ident(&input_struct_ident)?;

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
    Ok(method_input_structs)
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
                for (i, input) in m.sig.inputs.iter().enumerate() {
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
                        FnArg::Typed(argument_and_type) => {
                            let arg_index = if get_state.is_some() { i - 1 } else { i };
                            let arg_ident =
                                create_argument_ident(argument_and_type.pat.as_ref(), arg_index)?;

                            match_args.push(parse_quote! { #arg_ident });
                            dispatch_args.push(parse_quote! { input.#arg_ident });
                        }
                    }
                }

                // parse args
                let input_struct_ident = format_ident!("{}_{}_Input", bp_ident, ident);
                validate_type_ident(&input_struct_ident)?;
                stmts.push(parse_quote! {
                    let input: #input_struct_ident = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(args)).unwrap();
                });

                // load component state if needed
                if let Some(stmt) = get_state {
                    trace!("Generated stmt: {}", quote! { #stmt });
                    stmts.push(parse_quote! {
                        let mut component_data = ::scrypto::runtime::ComponentStatePointer::new();
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
                    quote! {
                        #[no_mangle]
                        pub extern "C" fn #fn_ident(args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                            use ::sbor::rust::ops::{Deref, DerefMut};

                            // Set up panic hook
                            ::scrypto::set_up_panic_hook();

                            #(#stmts)*
                        }
                    }
                };
                functions.push(extern_function);
            }
        };
    }

    Ok(functions)
}

fn create_argument_ident(argument: &Pat, index: usize) -> Result<Ident> {
    Ok(match argument {
        // If we have a standard parameter name - use that
        Pat::Ident(ident_pattern) => {
            let ident = if ident_pattern.ident.to_string().starts_with("_") {
                // Handle parameters starting with `_` - strip them to pass validation
                Ident::new(
                    ident_pattern.ident.to_string().trim_start_matches('_'),
                    ident_pattern.ident.span(),
                )
            } else {
                // Otherwise, just use the parameter as-is
                ident_pattern.ident.clone()
            };
            validate_field_ident(&ident)?;
            ident
        }
        // If it's not an ident, it's something more complicated (such as a destructuring), just use `argX`
        _ => format_ident!("arg{}", index, span = argument.span()),
    })
}

fn validate_type_ident(ident: &Ident) -> Result<()> {
    validate_type_name(&ident.to_string(), ident.span())
}

fn validate_type_name(name: &str, span: Span) -> Result<()> {
    sbor::validate_schema_type_name(name).map_err(|err| {
        Error::new(
            span,
            format!("Resultant identifier {} is invalid: {:?}", name, err),
        )
    })
}

fn validate_field_ident(ident: &Ident) -> Result<()> {
    validate_field_name(&ident.to_string(), ident.span())
}

fn validate_field_name(name: &str, span: Span) -> Result<()> {
    sbor::validate_schema_field_name(name).map_err(|err| {
        Error::new(
            span,
            format!("Resultant identifier {} is invalid: {:?}", name, err),
        )
    })
}

fn generate_stubs(
    component_ident: &Ident,
    functions_ident: &Ident,
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
                            FnArg::Receiver(r) => {
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
                            FnArg::Typed(argument_and_type) => {
                                let arg = create_argument_ident(
                                    argument_and_type.pat.as_ref(),
                                    input_len,
                                )?;
                                input_args.push(arg);

                                let ty = replace_self_with(&argument_and_type.ty, bp_ident);
                                input_types.push(ty);

                                input_len += 1;
                            }
                        }
                    }

                    let output = match &m.sig.output {
                        ReturnType::Default => parse_quote! { () },
                        ReturnType::Type(_, t) => replace_self_with(t, bp_ident),
                    };

                    if mutable.is_none() {
                        functions.push(parse_quote! {
                            fn #ident(#(#input_args: #input_types),*) -> #output {
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
                                self.call_raw(#name, scrypto_args!(
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
        #[derive(Clone, Copy, ::scrypto::prelude::ScryptoSbor)]
        pub struct #component_ident {
            pub handle: ::scrypto::component::ObjectStubHandle,
        }

        impl ::scrypto::component::ObjectStub for #component_ident {
            fn new(handle: ::scrypto::component::ObjectStubHandle) -> Self {
                Self {
                    handle
                }
            }
            fn handle(&self) -> &::scrypto::component::ObjectStubHandle {
                &self.handle
            }
        }

        impl #component_ident {
            #(#methods)*
        }

        pub trait #functions_ident {
            #(#functions)*
        }

        impl #functions_ident for ::scrypto::component::Blueprint<#bp_ident> {
        }
    };

    Ok(output)
}

fn get_restrict_to(attributes: &mut Vec<Attribute>) -> Option<String> {
    let to_remove = attributes.iter_mut().enumerate().find_map(
        |(index, attribute)| {
            if let Ok(Meta::List(meta_list)) = attribute.parse_meta() {
                if !meta_list.path.is_ident("restrict_to") {
                    return None;
                }

                if meta_list.nested.len() != 1 {
                    return None;
                }

                match meta_list.nested.first().unwrap() {
                    NestedMeta::Lit(Lit::Str(s)) => {
                        return Some((index, s.value()));
                    }
                    _ => {
                        return None;
                    }
                }
            }

            return None;
        }
    );

    if let Some((to_remove, authority)) = to_remove {
        attributes.remove(to_remove);
        Some(authority)
    } else {
        None
    }
}

#[allow(dead_code)]
struct GeneratedSchemaInfo {
    function_names: Vec<String>,
    function_schemas: Vec<Expr>,
    function_authority_names: Vec<String>,
    function_mapped_authorities: Vec<String>,
}

#[allow(dead_code)]
fn generate_schema(bp_ident: &Ident, items: &mut [ImplItem]) -> Result<GeneratedSchemaInfo> {
    let mut function_names = Vec::<String>::new();
    let mut function_schemas = Vec::<Expr>::new();
    let mut function_authority_names = Vec::<String>::new();
    let mut function_mapped_authorities = Vec::<String>::new();

    for item in items {
        trace!("Processing item: {}", quote! { #item });

        match item {
            ImplItem::Method(ref mut m) => {

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
                                    receiver = Some(
                                        quote! { ::scrypto::schema::ReceiverInfo::normal_ref_mut() },
                                    );
                                } else {
                                    receiver = Some(
                                        quote! { ::scrypto::schema::ReceiverInfo::normal_ref() },
                                    );
                                }
                            }
                            FnArg::Typed(_) => {}
                        }
                    }

                    let input_struct_ident = format_ident!("{}_{}_Input", bp_ident, m.sig.ident);
                    validate_type_ident(&input_struct_ident)?;
                    let output_type: Type = match &m.sig.output {
                        ReturnType::Default => parse_quote! {
                            ()
                        },
                        ReturnType::Type(_, t) => replace_self_with(t, bp_ident),
                    };
                    let export_name = format!("{}_{}", bp_ident, m.sig.ident);
                    validate_type_name(&export_name, bp_ident.span())?;

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
                        if let Some(authority) = get_restrict_to(&mut m.attrs) {
                            function_authority_names.push(function_name.clone());
                            function_mapped_authorities.push(authority);
                        }

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

    Ok(GeneratedSchemaInfo {
        function_names,
        function_schemas,
        function_authority_names,
        function_mapped_authorities,
    })
}

fn replace_self_with(t: &Type, name: &Ident) -> Type {
    match t {
        Type::Path(tp) => {
            let mut tp2 = tp.clone();
            tp2.path.segments.iter_mut().for_each(|s| {
                if s.ident == "Self" {
                    s.ident = name.clone()
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
    fn test_inconsistent_names_should_fail() {
        let input = TokenStream::from_str("struct A {} impl B { }").unwrap();
        assert!(matches!(handle_blueprint(input), Err(_)));
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

                    impl ::scrypto::component::ComponentState for Test {
                        const BLUEPRINT_NAME: &'static str = "Test";
                    }

                    impl HasStub for Test {
                        type Stub = TestObjectStub;
                    }

                    #[allow(non_camel_case_types)]
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct Test_x_Input { i : u32 }

                    #[allow(non_camel_case_types)]
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct Test_y_Input { i : u32 }

                    #[no_mangle]
                    pub extern "C" fn Test_x(args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                        use ::sbor::rust::ops::{Deref, DerefMut};

                        // Set up panic hook
                        ::scrypto::set_up_panic_hook();

                        let input: Test_x_Input = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(args)).unwrap();
                        let mut component_data = ::scrypto::runtime::ComponentStatePointer::new();
                        let state: DataRef<Test> = component_data.get();
                        let return_data = Test::x(state.deref(), input.i);
                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    #[no_mangle]
                    pub extern "C" fn Test_y(args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                        use ::sbor::rust::ops::{Deref, DerefMut};

                        // Set up panic hook
                        ::scrypto::set_up_panic_hook();

                        let input: Test_y_Input = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(args)).unwrap();
                        let return_data = Test::y(input.i);
                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    #[no_mangle]
                    pub extern "C" fn Test_schema() -> ::scrypto::engine::wasm_api::Slice {
                        use ::scrypto::schema::*;
                        use ::sbor::rust::prelude::*;
                        use ::sbor::schema::*;
                        use ::sbor::*;
                        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
                        let mut fields = Vec::new();
                        let type_index = aggregator.add_child_type_and_descendents::<Test>();
                        fields.push(type_index);
                        let mut functions = BTreeMap::new();
                        functions.insert(
                            "x".to_string(),
                            ::scrypto::schema::FunctionSchema {
                                receiver: Option::Some(::scrypto::schema::ReceiverInfo::normal_ref()),
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

                        let mut authority_schema = BTreeMap::new();

                        let mut method_authority_mapping = BTreeMap::new();

                        let return_data = BlueprintSchema {
                            outer_blueprint: None,
                            schema: generate_full_schema(aggregator),
                            fields,
                            collections: Vec::new(),
                            functions,
                            virtual_lazy_load_functions: BTreeMap::new(),
                            event_schema,
                            method_authority_mapping,
                            authority_schema,
                        };
                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    #[allow(non_camel_case_types)]
                    #[derive(Clone, Copy, ::scrypto::prelude::ScryptoSbor)]
                    pub struct TestObjectStub {
                        pub handle: ::scrypto::component::ObjectStubHandle,
                    }

                    impl ::scrypto::component::ObjectStub for TestObjectStub {
                        fn new(handle: ::scrypto::component::ObjectStubHandle) -> Self {
                            Self {
                                handle
                            }
                        }
                        fn handle(&self) -> &::scrypto::component::ObjectStubHandle {
                            &self.handle
                        }
                    }

                    impl TestObjectStub {
                        pub fn x(&self, i: u32) -> u32 {
                            self.call_raw("x", scrypto_args!(i))
                        }
                    }

                    pub trait TestFunctions {
                        fn y(i: u32) -> u32 {
                            ::scrypto::runtime::Runtime::call_function(::scrypto::runtime::Runtime::package_address(), "Test", "y", scrypto_args!(i))
                        }
                    }

                    impl TestFunctions for ::scrypto::component::Blueprint<Test> {}
                }
            },
        );
    }
}
