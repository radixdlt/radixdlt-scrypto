use crate::ast;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use radix_common::address::AddressBech32Decoder;
use regex::Regex;
use std::collections::BTreeMap;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::token::{Brace, Comma};
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

#[allow(dead_code)] // Fields for tokens from parse for completeness
pub struct GlobalComponent {
    pub ident: Ident,
    pub comma: Comma,
    pub address: Expr,
}

impl Parse for GlobalComponent {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let comma = input.parse()?;
        let address = input.parse()?;
        Ok(Self {
            ident,
            comma,
            address,
        })
    }
}

#[allow(dead_code)] // Fields for tokens from parse for completeness
pub struct ImportBlueprintFn {
    pub sig: Signature,
    pub semi_token: Token![;],
}

impl Parse for ImportBlueprintFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let sig = input.parse()?;
        let semi_token = input.parse()?;
        Ok(Self { sig, semi_token })
    }
}

#[allow(dead_code)] // Fields for tokens from parse for completeness
pub struct ImportBlueprint {
    pub package: Expr,
    pub comma0: Comma,
    pub blueprint: Ident,
    pub rename: Option<(Token![as], Ident)>,
    pub brace: Brace,
    pub functions: Vec<ImportBlueprintFn>,
}

impl Parse for ImportBlueprint {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let package = input.parse()?;
        let comma0 = input.parse()?;
        let blueprint = input.parse()?;
        let rename = if input.peek(Token![as]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };
        let brace = braced!(content in input);

        let functions = {
            let mut functions = Vec::new();
            while content.peek(Token![fn]) {
                functions.push(content.call(ImportBlueprintFn::parse)?)
            }
            functions
        };

        Ok(Self {
            package,
            comma0,
            blueprint,
            rename,
            brace,
            functions,
        })
    }
}

pub fn replace_macros_in_body(block: &mut Block, dependency_exprs: &mut Vec<Expr>) -> Result<()> {
    for stmt in &mut block.stmts {
        match stmt {
            Stmt::Expr(expr) => {
                replace_macros(expr, dependency_exprs)?;
            }
            Stmt::Semi(expr, ..) => {
                replace_macros(expr, dependency_exprs)?;
            }
            Stmt::Item(..) => {}
            Stmt::Local(local) => {
                if let Some((_, expr)) = &mut local.init {
                    replace_macros(expr, dependency_exprs)?;
                }
            }
        }
    }

    Ok(())
}

pub fn replace_macros(expr: &mut Expr, dependency_exprs: &mut Vec<Expr>) -> Result<()> {
    match expr {
        Expr::Macro(m) => {
            let macro_ident = m.mac.path.get_ident().unwrap();
            if macro_ident.eq(&Ident::new("global_component", Span::call_site())) {
                let global_component: GlobalComponent = m.mac.clone().parse_body()?;

                let blueprint = global_component.ident;
                let address = &global_component.address;
                let lit_str: LitStr = parse_quote!( #address );

                let (_hrp, _entity_type, address) =
                    AddressBech32Decoder::validate_and_decode_ignore_hrp(lit_str.value().as_str())
                        .unwrap();

                dependency_exprs.push(parse_quote! {
                    GlobalAddress :: new_or_panic([ #(#address),* ])
                });

                *expr = parse_quote! {
                    Global::<#blueprint>(#blueprint {
                        handle: ObjectStubHandle::Global(GlobalAddress :: new_or_panic([ #(#address),* ]))
                    })
                };
            } else if macro_ident.eq(&Ident::new("resource_manager", Span::call_site())) {
                let address: Expr = m.mac.clone().parse_body()?;
                match address {
                    Expr::Lit(..) => {
                        let lit_str: LitStr = parse_quote!( #address );
                        let (_hrp, _entity_type, address) =
                            AddressBech32Decoder::validate_and_decode_ignore_hrp(
                                lit_str.value().as_str(),
                            )
                            .unwrap();
                        dependency_exprs.push(parse_quote! {
                            GlobalAddress :: new_or_panic([ #(#address),* ])
                        });

                        *expr = parse_quote! {
                            ResourceManager::from_address(ResourceAddress :: new_or_panic([ #(#address),* ]))
                        };
                    }
                    _ => {
                        dependency_exprs.push(parse_quote! {
                            #address
                        });

                        *expr = parse_quote! {
                            ResourceManager::from_address(#address)
                        };
                    }
                };
            } else if macro_ident.eq(&Ident::new("package", Span::call_site())) {
                let address: Expr = m.mac.clone().parse_body()?;
                let lit_str: LitStr = parse_quote!( #address );

                let (_hrp, _entity_type, address) =
                    AddressBech32Decoder::validate_and_decode_ignore_hrp(lit_str.value().as_str())
                        .unwrap();

                dependency_exprs.push(parse_quote! {
                    PackageAddress :: new_or_panic([ #(#address),* ])
                });

                *expr = parse_quote! {
                    Global::<PackageStub>(PackageStub(ObjectStubHandle::Global(GlobalAddress :: new_or_panic([ #(#address),* ]))))
                };
            }
        }
        Expr::Tuple(tuple) => {
            for expr in &mut tuple.elems {
                replace_macros(expr, dependency_exprs)?;
            }
        }
        Expr::Assign(assign) => {
            replace_macros(assign.left.as_mut(), dependency_exprs)?;
            replace_macros(assign.right.as_mut(), dependency_exprs)?;
        }
        Expr::AssignOp(assign_op) => {
            replace_macros(assign_op.left.as_mut(), dependency_exprs)?;
            replace_macros(assign_op.right.as_mut(), dependency_exprs)?;
        }
        Expr::Type(ty) => {
            replace_macros(&mut ty.expr, dependency_exprs)?;
        }
        Expr::MethodCall(method_call) => {
            replace_macros(method_call.receiver.as_mut(), dependency_exprs)?;
            for expr in &mut method_call.args {
                replace_macros(expr, dependency_exprs)?;
            }
        }
        Expr::ForLoop(for_loop) => replace_macros_in_body(&mut for_loop.body, dependency_exprs)?,
        Expr::Call(call) => {
            replace_macros(call.func.as_mut(), dependency_exprs)?;
            for expr in &mut call.args {
                replace_macros(expr, dependency_exprs)?;
            }
        }
        Expr::Reference(reference) => {
            replace_macros(reference.expr.as_mut(), dependency_exprs)?;
        }
        Expr::Array(array) => {
            for expr in &mut array.elems {
                replace_macros(expr, dependency_exprs)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Derive a sensible type identifier from a path, so to pass Radix Engine's constraints.
pub fn derive_sensible_identifier_from_path(path: &Path) -> Result<String> {
    if let Some(segment) = path.segments.last() {
        let mut result = String::new();
        let mut with_underscore = false;

        for c in quote! { #segment }.to_string().chars() {
            if c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c >= '0' && c <= '9' {
                result.push(c);
                with_underscore = false;
            } else {
                if !with_underscore {
                    result.push('_');
                    with_underscore = true;
                }
            }
        }

        // Trim
        if result.len() > 1 && result.ends_with('_') {
            result.pop();
        }

        Ok(result)
    } else {
        Err(Error::new(
            path.span(),
            "Failed to derive sensible identifier",
        ))
    }
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

        let contains_super_import = use_statements
            .iter()
            .map(|x| quote! { #(#x) }.to_string())
            .any(|x| x.contains("super :: *"));
        if !contains_super_import {
            let item: ItemUse = parse_quote! { use super::*; };
            use_statements.push(item);
        }

        let contains_prelude_import = use_statements
            .iter()
            .map(|x| quote! { #(#x) }.to_string())
            .any(|x| x.contains("scrypto :: prelude :: *"));
        if !contains_prelude_import {
            let item: ItemUse = parse_quote! { use scrypto::prelude::*; };
            use_statements.push(item);
        }

        {
            let item: ItemUse = parse_quote! { use scrypto::prelude::MethodAccessibility::*; };
            use_statements.push(item);
            let item: ItemUse = parse_quote! { use scrypto::prelude::RoyaltyAmount::*; };
            use_statements.push(item);
        }

        use_statements
    };

    let mut dependency_exprs = Vec::new();

    let const_statements = {
        let mut const_statements = bp.const_statements.clone();

        // Bech32 decode constant dependencies
        for item in &mut const_statements {
            let ty = item.ty.as_ref();
            let ident = item.ident.clone();
            let type_string = quote! { #ty }.to_string();
            if type_string.contains("ResourceAddress")
                || type_string.contains("ComponentAddress")
                || type_string.contains("PackageAddress")
                || type_string.contains("GlobalAddress")
            {
                match item.expr.as_mut() {
                    Expr::Macro(m) => {
                        if !m
                            .mac
                            .path
                            .get_ident()
                            .unwrap()
                            .eq(&Ident::new("address", Span::call_site()))
                        {
                            continue;
                        }

                        let tokens = &m.mac.tokens;
                        let lit_str: LitStr = parse_quote!( #tokens );

                        let (_hrp, _entity_type, address) =
                            AddressBech32Decoder::validate_and_decode_ignore_hrp(
                                lit_str.value().as_str(),
                            )
                            .unwrap();

                        let expr: Expr = parse_quote! {
                            #ty :: new_or_panic([ #(#address),* ])
                        };
                        item.expr = Box::new(expr);
                    }
                    _ => {}
                }

                let expr: Expr = parse_quote! { #ident };
                dependency_exprs.push(expr);
            } else {
                replace_macros(item.expr.as_mut(), &mut dependency_exprs)?;
            }
        }

        const_statements
    };

    let generated_schema_info = generate_schema(bp_ident, bp_items, &mut dependency_exprs)?;
    let fn_idents = generated_schema_info.fn_idents;
    let method_idents = generated_schema_info.method_idents;
    let function_idents = generated_schema_info.function_idents;

    let blueprint_name = bp_ident.to_string();
    let owned_typed_name = format!("Owned{}", blueprint_name);
    let global_typed_name = format!("Global{}", blueprint_name);

    let mut macro_statements = bp.macro_statements;

    let method_auth_statements = {
        let method_auth_index = macro_statements.iter().position(|item| {
            item.mac
                .path
                .get_ident()
                .unwrap()
                .eq(&Ident::new("enable_method_auth", Span::call_site()))
        });
        if let Some(method_auth_index) = method_auth_index {
            let auth_macro = macro_statements.remove(method_auth_index);
            quote! {
                #auth_macro
            }
        } else {
            quote! {
                fn method_auth_template() -> scrypto::blueprints::package::MethodAuthTemplate {
                    scrypto::blueprints::package::MethodAuthTemplate::AllowAll
                }
            }
        }
    };

    let import_statements = {
        let mut import_statements = Vec::new();
        loop {
            let import_blueprint_index = macro_statements.iter().position(|item| {
                item.mac
                    .path
                    .get_ident()
                    .unwrap()
                    .eq(&Ident::new("extern_blueprint", Span::call_site()))
            });
            if let Some(import_blueprint_index) = import_blueprint_index {
                let import_macro = macro_statements.remove(import_blueprint_index);
                let import_blueprint: ImportBlueprint = import_macro.mac.parse_body()?;

                let package_expr = {
                    let package = import_blueprint.package;
                    match package {
                        Expr::Lit(..) => {
                            let lit_str: LitStr = parse_quote!( #package );
                            let (_hrp, _entity_type, address) =
                                AddressBech32Decoder::validate_and_decode_ignore_hrp(
                                    lit_str.value().as_str(),
                                )
                                .unwrap();
                            let package_expr: Expr = parse_quote! {
                                PackageAddress :: new_or_panic([ #(#address),* ])
                            };
                            package_expr
                        }
                        _ => package,
                    }
                };

                dependency_exprs.push(package_expr.clone());

                let blueprint_name = import_blueprint.blueprint.to_string();
                let blueprint = if let Some((_, rename)) = import_blueprint.rename {
                    rename
                } else {
                    import_blueprint.blueprint
                };

                let owned_typed_name = format!("Owned{}", blueprint);
                let global_typed_name = format!("Global{}", blueprint);
                let blueprint_functions_ident = format_ident!("{}Functions", blueprint);

                let mut methods = Vec::new();
                let mut functions = Vec::new();
                for function in import_blueprint.functions {
                    let is_method = function
                        .sig
                        .inputs
                        .iter()
                        .find(|arg| matches!(arg, FnArg::Receiver(..)))
                        .is_some();
                    if is_method {
                        methods.push(function.sig);
                    } else {
                        functions.push(function.sig);
                    }
                }

                let import_statement = quote! {
                    extern_blueprint_internal! {
                        #package_expr,
                        #blueprint,
                        #blueprint_name,
                        #owned_typed_name,
                        #global_typed_name,
                        #blueprint_functions_ident {
                            #(#functions;)*
                        },
                        {
                            #(#methods;)*
                        }
                    }
                };
                import_statements.push(import_statement);
            } else {
                break;
            }
        }
        import_statements
    };

    let registered_type_ident = format_ident!("{}RegisteredType", bp_ident);
    let mut registered_type_traits = Vec::<ItemTrait>::new();
    let mut registered_type_structs = Vec::<ItemStruct>::new();
    let mut registered_type_impls = Vec::<ItemImpl>::new();
    let mut registered_type_paths = BTreeMap::<String, Path>::new();

    // Register type trait
    registered_type_traits.push(parse_quote! {
        pub trait #registered_type_ident {
            fn blueprint_type_identifier() -> BlueprintTypeIdentifier;
        }
    });

    // KeyValueStore trait and impl
    let registered_type_kv_store = format_ident!("{}KeyValueStore", bp_ident);
    registered_type_traits.push(parse_quote! {
        pub trait #registered_type_kv_store {
            fn new_with_registered_type() -> Self;
        }
    });
    registered_type_impls.push(parse_quote! {
        impl<
            K: ScryptoEncode + ScryptoDecode + ScryptoDescribe + #registered_type_ident,
            V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + #registered_type_ident,
        > #registered_type_kv_store for KeyValueStore<K, V> {
            fn new_with_registered_type() -> Self {
                let store_schema = RemoteKeyValueStoreDataSchema {
                    key_type: K::blueprint_type_identifier(),
                    value_type: V::blueprint_type_identifier(),
                    allow_ownership: true,
                };

                Self {
                    id: Own(ScryptoVmV1Api::kv_store_new(SborFixedEnumVariant::<
                        KV_STORE_DATA_SCHEMA_VARIANT_REMOTE,
                        RemoteKeyValueStoreDataSchema,
                    > {
                        fields: store_schema,
                    })),
                    key: PhantomData,
                    value: PhantomData,
                }
            }
        }
    });

    // ResourceBuilder trait and impl
    let registered_type_resource_builder = format_ident!("{}ResourceBuilder", bp_ident);
    registered_type_traits.push(parse_quote! {
        pub trait #registered_type_resource_builder {
            fn new_string_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<StringNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>>;

            fn new_integer_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<IntegerNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>>;

            fn new_bytes_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<BytesNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>>;

            fn new_ruid_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<RUIDNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>>;
        }
    });
    registered_type_impls.push(parse_quote! {
        impl #registered_type_resource_builder for ResourceBuilder {
            fn new_string_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<StringNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>> {
                InProgressResourceBuilder::new(
                    owner_role,
                    NonFungibleResourceType::new(SborFixedEnumVariant {
                        fields: RemoteNonFungibleDataSchema::new(
                            D::blueprint_type_identifier(),
                            D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                        ),
                    }),
                )
            }

            fn new_integer_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<IntegerNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>> {
                InProgressResourceBuilder::new(
                    owner_role,
                    NonFungibleResourceType::new(SborFixedEnumVariant {
                        fields: RemoteNonFungibleDataSchema::new(
                            D::blueprint_type_identifier(),
                            D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                        ),
                    }),
                )
            }

            fn new_bytes_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<BytesNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>> {
                InProgressResourceBuilder::new(
                    owner_role,
                    NonFungibleResourceType::new(SborFixedEnumVariant {
                        fields: RemoteNonFungibleDataSchema::new(
                            D::blueprint_type_identifier(),
                            D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                        ),
                    }),
                )
            }

            fn new_ruid_non_fungible_with_registered_type<D: NonFungibleData + #registered_type_ident>(
                owner_role: OwnerRole,
            ) -> InProgressResourceBuilder<NonFungibleResourceType<RUIDNonFungibleLocalId, D, SborFixedEnumVariant<NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE, RemoteNonFungibleDataSchema>>> {
                InProgressResourceBuilder::new(
                    owner_role,
                    NonFungibleResourceType::new(SborFixedEnumVariant {
                        fields: RemoteNonFungibleDataSchema::new(
                            D::blueprint_type_identifier(),
                            D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                        ),
                    }),
                )
            }
        }
    });

    for attribute in &blueprint.attributes {
        if attribute.path.is_ident("types") {
            let types_inner = parse2::<ast::TypesInner>(attribute.tokens.clone())?;
            for aliasable_type in types_inner.aliasable_types {
                let path = aliasable_type.path;
                let type_name;
                if let Some(alias) = aliasable_type.alias {
                    type_name = alias.to_string();
                    registered_type_structs.push(parse_quote! {
                        #[derive(ScryptoSbor)]
                        #[sbor(transparent)]
                        pub struct #alias(#path);
                    });
                    registered_type_impls.push(parse_quote! {
                        impl From<#path> for #alias {
                            fn from(value: #path) -> Self {
                                Self(value)
                            }
                        }
                    });
                    registered_type_impls.push(parse_quote! {
                        impl From<#alias> for #path {
                            fn from(value: #alias) -> Self {
                                value.0
                            }
                        }
                    });
                    registered_type_impls.push(parse_quote! {
                        impl #registered_type_ident for #alias {
                            fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                                BlueprintTypeIdentifier {
                                    package_address: Runtime::package_address(),
                                    blueprint_name: #bp_name.to_owned(),
                                    type_name: #type_name.to_owned(),
                                }
                            }
                        }
                    });
                } else {
                    type_name = derive_sensible_identifier_from_path(&path)?;
                    registered_type_impls.push(parse_quote! {
                        impl #registered_type_ident for #path {
                            fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                                BlueprintTypeIdentifier {
                                    package_address: Runtime::package_address(),
                                    blueprint_name: #bp_name.to_owned(),
                                    type_name: #type_name.to_owned(),
                                }
                            }
                        }
                    });
                }

                if let Some(..) = registered_type_paths.insert(type_name, path.clone()) {
                    return Err(Error::new(
                        path.span(),
                        "A type with an identical name has already been registered",
                    ));
                }
            }
        }
    }

    #[cfg(feature = "no-schema")]
    let output_schema = quote! {};
    #[cfg(not(feature = "no-schema"))]
    let output_schema = {
        let function_auth_statements = {
            let function_auth_index = macro_statements.iter().position(|item| {
                item.mac
                    .path
                    .get_ident()
                    .unwrap()
                    .eq(&Ident::new("enable_function_auth", Span::call_site()))
            });
            if let Some(function_auth_index) = function_auth_index {
                let auth_macro = macro_statements.remove(function_auth_index);
                quote! {
                    #auth_macro
                }
            } else {
                quote! {
                    fn function_auth() -> scrypto::blueprints::package::FunctionAuth {
                        scrypto::blueprints::package::FunctionAuth::AllowAll
                    }
                }
            }
        };

        let package_royalties_statements = {
            let package_royalties_index = macro_statements.iter().position(|item| {
                item.mac
                    .path
                    .get_ident()
                    .unwrap()
                    .eq(&Ident::new("enable_package_royalties", Span::call_site()))
            });
            if let Some(package_royalties_index) = package_royalties_index {
                let royalties_macro = macro_statements.remove(package_royalties_index);
                quote! {
                    #royalties_macro
                }
            } else {
                quote! {
                    fn package_royalty_config() -> PackageRoyaltyConfig {
                        PackageRoyaltyConfig::Disabled
                    }
                }
            }
        };

        let schema_ident = format_ident!("{}_schema", bp_ident);
        let fn_names = generated_schema_info.fn_names;
        let fn_schemas = generated_schema_info.fn_schemas;

        // Getting the event types and other named types from attribute
        let (event_type_names, event_type_paths, registered_type_names, registered_type_paths) = {
            let mut event_type_paths = BTreeMap::<String, Path>::new();
            for attribute in blueprint.attributes {
                if attribute.path.is_ident("events") {
                    let events_inner = parse2::<ast::EventsInner>(attribute.tokens.clone())?;
                    for path in events_inner.paths.iter() {
                        let type_name = derive_sensible_identifier_from_path(&path)?;
                        if let Some(..) = event_type_paths.insert(type_name, path.clone()) {
                            return Err(Error::new(
                                path.span(),
                                "An event with an identical name has already been named",
                            ));
                        }
                    }
                } else if attribute.path.is_ident("types") {
                }
                // None of the attributes to apply at the top-level of blueprint macros matched. So,
                // we provide an error to the user that they're using an incorrect attribute macro
                // that has no effect.
                else {
                    return Err(Error::new(
                        attribute.path.span(),
                        format!("Attribute is invalid for blueprints."),
                    ));
                }
            }
            (
                event_type_paths
                    .keys()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                event_type_paths
                    .values()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                registered_type_paths
                    .keys()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                registered_type_paths
                    .values()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        };

        quote! {
            #function_auth_statements

            #package_royalties_statements

            #[no_mangle]
            pub extern "C" fn #schema_ident() -> ::scrypto::engine::wasm_api::Slice {
                use ::scrypto::radix_blueprint_schema_init::*;
                use sbor::rust::prelude::*;
                use sbor::schema::*;
                use sbor::*;

                let schema = {
                    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

                    // Aggregate fields
                    let mut fields = Vec::new();
                    let type_index = aggregator.add_child_type_and_descendents::<#bp_ident>();
                    fields.push(FieldSchema::static_field(type_index));

                    let state = BlueprintStateSchemaInit {
                        fields,
                        collections: Vec::new(),
                    };

                    // Aggregate functions
                    let functions = {
                        let mut functions: IndexMap<String, FunctionSchemaInit> = index_map_new();
                        #(
                            functions.insert(#fn_names.to_string(), #fn_schemas);
                        )*

                        BlueprintFunctionsSchemaInit {
                            functions,
                        }
                    };

                    // Aggregate event schemas
                    let events = {
                        let mut event_schema = index_map_new();
                        #({
                            let local_type_index = aggregator.add_child_type_and_descendents::<#event_type_paths>();
                            event_schema.insert(#event_type_names.to_owned(), TypeRef::Static(local_type_index));
                        })*
                        BlueprintEventSchemaInit {
                            event_schema,
                        }
                    };

                    // Aggregate registered types
                    let types = {
                        let mut type_schema = index_map_new();
                        #({
                            let local_type_index = aggregator.add_child_type_and_descendents::<#registered_type_paths>();
                            type_schema.insert(#registered_type_names.to_owned(), local_type_index);
                        })*
                        BlueprintTypeSchemaInit {
                            type_schema
                        }
                    };

                    let schema = generate_full_schema(aggregator);

                    BlueprintSchemaInit {
                        generics: vec![],
                        schema,
                        state,
                        events,
                        types,
                        functions,
                        hooks: BlueprintHooksInit::default(),
                    }
                };

                let mut dependencies = index_set_new();
                #({
                    dependencies.insert(#dependency_exprs.into());
                })*

                let auth_config = {
                    scrypto::blueprints::package::AuthConfig {
                        method_auth: method_auth_template(),
                        function_auth: function_auth(),
                    }
                };

                let royalty_config = package_royalty_config();

                let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
                    blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
                    is_transient: false,
                    feature_set: IndexSet::default(),
                    dependencies,
                    schema,
                    auth_config,
                    royalty_config,
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

        impl HasMethods for #bp_ident {
            type Permissions = Methods<MethodAccessibility>;
            type Royalties = Methods<(RoyaltyAmount, bool)>;
        }

        impl HasTypeInfo for #bp_ident {
            const PACKAGE_ADDRESS: Option<PackageAddress> = None;
            const BLUEPRINT_NAME: &'static str = #blueprint_name;
            const OWNED_TYPE_NAME: &'static str = #owned_typed_name;
            const GLOBAL_TYPE_NAME: &'static str = #global_typed_name;
        }
    };

    let methods_struct = generate_methods_struct(method_idents);
    let functions_struct = generate_functions_struct(function_idents);
    let fns_struct = generate_fns_struct(fn_idents);

    trace!("Generated mod: \n{}", quote! { #output_original_code });
    let method_input_structs = generate_method_input_structs(bp_ident, bp_items)?;

    let functions = generate_dispatcher(bp_ident, bp_items)?;
    let output_dispatcher = quote! {
        #(#method_input_structs)*
        #(#functions)*
    };

    trace!("Generated dispatcher: \n{}", quote! { #output_dispatcher });

    let output_stubs = generate_stubs(&stub_ident, &functions_ident, bp_ident, bp_items)?;

    let output_test_bindings_struct = quote! {
        #[derive(Debug, Clone, Copy)]
        pub struct #bp_ident(pub NodeId);
    };

    let output_test_struct_impls = quote! {
        impl<D: sbor::Decoder<::scrypto::prelude::ScryptoCustomValueKind>>
            ::scrypto::prelude::Decode<::scrypto::prelude::ScryptoCustomValueKind, D>
            for #bp_ident
        {
            #[inline]
            fn decode_body_with_value_kind(
                decoder: &mut D,
                value_kind: ::scrypto::prelude::ValueKind<
                    ::scrypto::prelude::ScryptoCustomValueKind,
                >,
            ) -> Result<Self, ::scrypto::prelude::DecodeError> {
                let node_id = match value_kind {
                    ValueKind::Custom(::scrypto::prelude::ScryptoCustomValueKind::Reference) => {
                        <::scrypto::prelude::Reference as ::scrypto::prelude::Decode<
                            ::scrypto::prelude::ScryptoCustomValueKind,
                            D,
                        >>::decode_body_with_value_kind(decoder, value_kind)
                        .map(|reference| reference.0)
                    }
                    ValueKind::Custom(::scrypto::prelude::ScryptoCustomValueKind::Own) => {
                        <::scrypto::prelude::Own as ::scrypto::prelude::Decode<
                            ::scrypto::prelude::ScryptoCustomValueKind,
                            D,
                        >>::decode_body_with_value_kind(decoder, value_kind)
                        .map(|own| own.0)
                    }
                    _ => Err(::scrypto::prelude::DecodeError::InvalidCustomValue),
                }?;
                Ok(Self(node_id))
            }
        }

        impl ::core::convert::TryFrom<#bp_ident> for ::scrypto::prelude::ComponentAddress {
            type Error = ::scrypto::prelude::ParseComponentAddressError;

            fn try_from(value: #bp_ident) -> Result<Self, Self::Error> {
                ::scrypto::prelude::ComponentAddress::try_from(value.0)
            }
        }
        impl ::core::convert::TryFrom<#bp_ident> for ::scrypto::prelude::ResourceAddress {
            type Error = ::scrypto::prelude::ParseResourceAddressError;

            fn try_from(value: #bp_ident) -> Result<Self, Self::Error> {
                ::scrypto::prelude::ResourceAddress::try_from(value.0)
            }
        }
        impl ::core::convert::TryFrom<#bp_ident> for ::scrypto::prelude::PackageAddress {
            type Error = ::scrypto::prelude::ParsePackageAddressError;

            fn try_from(value: #bp_ident) -> Result<Self, Self::Error> {
                ::scrypto::prelude::PackageAddress::try_from(value.0)
            }
        }
        impl ::core::convert::TryFrom<#bp_ident> for ::scrypto::prelude::GlobalAddress {
            type Error = ::scrypto::prelude::ParseGlobalAddressError;

            fn try_from(value: #bp_ident) -> Result<Self, Self::Error> {
                ::scrypto::prelude::GlobalAddress::try_from(value.0)
            }
        }
        impl ::core::convert::TryFrom<#bp_ident> for ::scrypto::prelude::InternalAddress {
            type Error = ::scrypto::prelude::ParseInternalAddressError;

            fn try_from(value: #bp_ident) -> Result<Self, Self::Error> {
                ::scrypto::prelude::InternalAddress::try_from(value.0)
            }
        }
        impl ::core::convert::From<#bp_ident> for ::scrypto::prelude::Own {
            fn from(value: #bp_ident) -> Self {
                Self(value.0)
            }
        }
        impl ::core::convert::From<#bp_ident> for ::scrypto::prelude::Reference {
            fn from(value: #bp_ident) -> Self {
                Self(value.0)
            }
        }
        impl ::core::convert::From<#bp_ident> for ::scrypto::prelude::NodeId {
            fn from(value: #bp_ident) -> ::scrypto::prelude::NodeId {
                value.0
            }
        }
    };

    let test_bindings_mod_ident = format_ident!("{}_test", module_ident);

    let output_test_bindings_fns = {
        let fns = generate_test_bindings_fns(&bp_name, bp_items)?;

        quote! {
            impl #bp_ident {
                #(#fns)*
            }
        }
    };

    let output_test_bindings_state = generate_test_bindings_state(bp_strut);

    let output = quote! {
        #[automatically_derived]
        pub mod #module_ident {
            #(#use_statements)*

            #(#const_statements)*

            #(#import_statements)*

            #(#macro_statements)*

            #method_auth_statements

            #output_original_code

            #methods_struct

            #functions_struct

            #fns_struct

            #output_dispatcher

            #output_schema

            #output_stubs

            #(#registered_type_traits)*

            #(#registered_type_structs)*

            #(#registered_type_impls)*
        }

        #[automatically_derived]
        pub mod #test_bindings_mod_ident {
            #(#use_statements)*

            #output_test_bindings_state

            #output_test_bindings_struct

            #output_test_struct_impls

            #output_test_bindings_fns
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("blueprint", &output);

    trace!("handle_blueprint() finishes");
    Ok(output)
}

fn generate_methods_struct(method_idents: Vec<Ident>) -> TokenStream {
    let method_names: Vec<String> = method_idents.iter().map(|i| i.to_string()).collect();

    if method_idents.is_empty() {
        quote! {
            pub struct Methods<T> {
                t: PhantomData<T>,
            }

            impl<T> MethodMapping<T> for Methods<T> {
                const MODULE_ID: scrypto::api::ModuleId = scrypto::api::ModuleId::Main;

                fn to_mapping(self) -> Vec<(String, T)> {
                    vec![]
                }


                fn methods() -> Vec<&'static str> {
                    vec![]
                }
            }
        }
    } else {
        quote! {
            pub struct Methods<T> {
                #(
                    #method_idents: T,
                )*
            }

            impl<T> MethodMapping<T> for Methods<T> {
                const MODULE_ID: scrypto::api::ModuleId = scrypto::api::ModuleId::Main;

                fn to_mapping(self) -> Vec<(String, T)> {
                    vec![
                        #(
                            (#method_names.to_string(), self.#method_idents)
                        ,
                        )*
                    ]
                }

                fn methods() -> Vec<&'static str> {
                    vec![ #(#method_names,)* ]
                }
            }
        }
    }
}

fn generate_functions_struct(function_idents: Vec<Ident>) -> TokenStream {
    let function_names: Vec<String> = function_idents.iter().map(|i| i.to_string()).collect();

    if function_idents.is_empty() {
        quote! {
            pub struct Functions<T> {
                t: PhantomData<T>,
            }

            impl<T> FnMapping<T> for Functions<T> {
                fn to_mapping(self) -> Vec<(String, T)> {
                    vec![]
                }
            }
        }
    } else {
        quote! {
            pub struct Functions<T> {
                #(
                    #function_idents: T,
                )*
            }

            impl<T> FnMapping<T> for Functions<T> {
                fn to_mapping(self) -> Vec<(String, T)> {
                    vec![
                        #(
                            (#function_names.to_string(), self.#function_idents)
                        ,
                        )*
                    ]
                }
            }
        }
    }
}

fn generate_fns_struct(fn_idents: Vec<Ident>) -> TokenStream {
    let fn_names: Vec<String> = fn_idents.iter().map(|i| i.to_string()).collect();

    if fn_idents.is_empty() {
        quote! {
            pub struct Fns<T> {
                t: PhantomData<T>,
            }

            impl<T> FnMapping<T> for Fns<T> {
                fn to_mapping(self) -> Vec<(String, T)> {
                    vec![]
                }
            }
        }
    } else {
        quote! {
            pub struct Fns<T> {
                #(
                    #fn_idents: T,
                )*
            }

            impl<T> FnMapping<T> for Fns<T> {
                fn to_mapping(self) -> Vec<(String, T)> {
                    vec![
                        #(
                            (#fn_names.to_string(), self.#fn_idents)
                        ,
                        )*
                    ]
                }
            }
        }
    }
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
                            use sbor::rust::ops::{Deref, DerefMut};

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
            let ident = if ident_pattern.ident.to_string().starts_with('_') {
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
    let mut functions = Vec::<ImplItem>::new();
    let mut function_traits = Vec::<ImplItem>::new();
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
                        function_traits.push(parse_quote! {
                            fn #ident(#(#input_args: #input_types),*) -> #output;
                        });
                        functions.push(parse_quote! {
                            fn #ident(#(#input_args: #input_types),*) -> #output {
                                Self::call_function_raw(#name, scrypto_args!(#(#input_args),*))
                            }
                        });
                    } else {
                        methods.push(parse_quote! {
                            pub fn #ident(&self #(, #input_args: #input_types)*) -> #output {
                                self.call_raw(#name, scrypto_args!(#(#input_args),*))
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
            type AddressType = ComponentAddress;

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
            #(#function_traits)*
        }

        impl #functions_ident for ::scrypto::component::Blueprint<#bp_ident> {
            #(#functions)*
        }
    };

    Ok(output)
}

#[allow(dead_code)]
struct GeneratedSchemaInfo {
    fn_names: Vec<String>,
    fn_schemas: Vec<Expr>,
    fn_idents: Vec<Ident>,
    method_idents: Vec<Ident>,
    function_idents: Vec<Ident>,
}

#[allow(dead_code)]
fn generate_schema(
    bp_ident: &Ident,
    items: &mut [ImplItem],
    dependency_exprs: &mut Vec<Expr>,
) -> Result<GeneratedSchemaInfo> {
    let mut fn_names = Vec::<String>::new();
    let mut fn_schemas = Vec::<Expr>::new();
    let mut fn_idents = Vec::<Ident>::new();
    let mut method_idents = Vec::<Ident>::new();
    let mut function_idents = Vec::<Ident>::new();

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
                                        quote! { ::scrypto::radix_blueprint_schema_init::ReceiverInfo::normal_ref_mut() },
                                    );
                                } else {
                                    receiver = Some(
                                        quote! { ::scrypto::radix_blueprint_schema_init::ReceiverInfo::normal_ref() },
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

                    replace_macros_in_body(&mut m.block, dependency_exprs)?;

                    fn_names.push(function_name);
                    fn_idents.push(m.sig.ident.clone());

                    if receiver.is_none() {
                        function_idents.push(m.sig.ident.clone());
                        fn_schemas.push(parse_quote! {
                            FunctionSchemaInit {
                                receiver: Option::None,
                                input: TypeRef::Static(aggregator.add_child_type_and_descendents::<#input_struct_ident>()),
                                output: TypeRef::Static(aggregator.add_child_type_and_descendents::<#output_type>()),
                                export: #export_name.to_string(),
                            }
                        });
                    } else {
                        method_idents.push(m.sig.ident.clone());
                        fn_schemas.push(parse_quote! {
                            FunctionSchemaInit {
                                receiver: Option::Some(#receiver),
                                input: TypeRef::Static(aggregator.add_child_type_and_descendents::<#input_struct_ident>()),
                                output: TypeRef::Static(aggregator.add_child_type_and_descendents::<#output_type>()),
                                export: #export_name.to_string(),
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
        fn_names,
        fn_schemas,
        fn_idents,
        method_idents,
        function_idents,
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

fn generate_test_bindings_fns(bp_name: &str, items: &[ImplItem]) -> Result<Vec<TokenStream>> {
    let mut functions = Vec::new();

    for item in items {
        if let ImplItem::Method(ref impl_item) = item {
            // Ignore any method that is not public - it doesn't need to have test bindings for it.
            match impl_item.vis {
                Visibility::Public(..) => {}
                _ => continue,
            }
            let func_ident = &impl_item.sig.ident;

            // A flag that keeps track of whether this is a function or a method by tracking whether
            // a receiver has been encountered or not.
            let mut is_receiver_encountered = false;
            let mut args = Vec::<TokenStream>::new();
            let mut arg_idents = Vec::<Ident>::new();
            for (i, arg) in impl_item.sig.inputs.iter().enumerate() {
                match arg {
                    FnArg::Receiver(receiver) => {
                        args.push(quote!(#receiver));
                        is_receiver_encountered = true;
                    }
                    FnArg::Typed(typed) => {
                        let arg_ident = match typed.pat.as_ref() {
                            Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                            _ => parse2(
                                TokenStream::from_str(&format!("unnamed_argument_index_{}", i))
                                    .unwrap(),
                            )?,
                        };
                        let arg_type = type_replacement(typed.ty.as_ref(), None);

                        args.push(quote! { #arg_ident: #arg_type });
                        arg_idents.push(arg_ident.clone());
                    }
                }
            }

            if !is_receiver_encountered {
                args.push(quote! { blueprint_package_address: ::scrypto::prelude::PackageAddress })
            }
            args.push(quote! { env: &mut Y });

            let invocation_type = TokenStream::from_str(if is_receiver_encountered {
                "call_method"
            } else {
                "call_function"
            })?;
            let invocation_args = {
                if !is_receiver_encountered {
                    quote! { blueprint_package_address, #bp_name, stringify!(#func_ident), ::scrypto::prelude::scrypto_encode(&( #(#arg_idents,)* )).unwrap() }
                } else {
                    quote! { &self.0, stringify!(#func_ident), ::scrypto::prelude::scrypto_encode(&( #(#arg_idents,)* )).unwrap() }
                }
            };

            let rtn_type = match &impl_item.sig.output {
                ReturnType::Default => parse_quote!(()),
                ReturnType::Type(_, rtn_type) => type_replacement(rtn_type.as_ref(), Some(bp_name)),
            };

            let func = quote! {
                pub fn #func_ident<Y: SystemApi<E>, E: SystemApiError> (
                    #(#args),*
                ) -> Result<#rtn_type, E> {
                    let rtn = env. #invocation_type ( #invocation_args )?;
                    Ok(::scrypto::prelude::scrypto_decode(&rtn).unwrap())
                }
            };
            functions.push(func);
        }
    }
    Ok(functions)
}

fn generate_test_bindings_state(bp_struct: &ItemStruct) -> ItemStruct {
    let mut bp_struct = bp_struct.clone();
    bp_struct.ident = format_ident!("{}State", bp_struct.ident);

    match &mut bp_struct.fields {
        Fields::Unit => {}
        Fields::Named(named) => named.named.iter_mut().for_each(|Field { ty, vis, .. }| {
            *ty = type_replacement(ty, Some(bp_struct.ident.to_string().as_str()));
            *vis = Visibility::Public(VisPublic {
                pub_token: token::Pub::default(),
            });
        }),
        Fields::Unnamed(unnamed) => {
            unnamed
                .unnamed
                .iter_mut()
                .for_each(|Field { ty, vis, .. }| {
                    *ty = type_replacement(ty, Some(bp_struct.ident.to_string().as_str()));
                    *vis = Visibility::Public(VisPublic {
                        pub_token: token::Pub::default(),
                    });
                })
        }
    };

    bp_struct.attrs = vec![parse_quote! { #[derive(::scrypto::prelude::ScryptoSbor)] }];
    bp_struct.vis = Visibility::Public(VisPublic {
        pub_token: token::Pub::default(),
    });

    bp_struct
}

/// This function performs the following replacement and is used for the function returns:
///
/// Before:
/// ```rust,ignore
/// fn instantiate() -> (
///     Global<ThisBlueprint>,
///     Global<AnotherBlueprint>,
///     Owned<SomeOtherBlueprint>
/// );
/// ```
/// After
/// ```rust,ignore
/// fn instantiate() -> (
///     Self,
///     ::scrypto::prelude::GlobalAddress,
///     ::scrypto::prelude::InternalAddress
/// );
/// ```
///
/// Thus, each `Global<ThisBlueprint>` or `Owned<ThisBlueprint>` is replaced with `Self` and other
/// `Global<T>` and `Owned<T>` are replaced with address types.
fn type_replacement(ty: &Type, bp_name: Option<&str>) -> Type {
    let global_pattern = Regex::new(r"Global<.+?>").unwrap();
    let owned_pattern = Regex::new(r"Owned<.+?>").unwrap();

    let mut str = ty.to_token_stream().to_string().replace(' ', "");
    if let Some(bp_name) = bp_name {
        str = str.replace(format!("Owned<{}>", bp_name).as_str(), "Self");
        str = str.replace(format!("Global<{}>", bp_name).as_str(), "Self");
    }
    str = global_pattern
        .replace_all(str.as_str(), "::scrypto::prelude::Reference")
        .into_owned();
    str = owned_pattern
        .replace_all(str.as_str(), "::scrypto::prelude::Own")
        .into_owned();

    parse2(proc_macro2::TokenStream::from_str(str.as_str()).unwrap()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::print_generated_code;
    use proc_macro2::TokenStream;
    use radix_common::prelude::*;
    use std::str::FromStr;

    fn assert_code_eq(actual: TokenStream, expected: TokenStream) {
        print_generated_code("Actual", &actual);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_inconsistent_names_should_fail() {
        let input = TokenStream::from_str("struct A {} impl B { }").unwrap();
        assert_matches!(handle_blueprint(input), Err(_));
    }

    #[test]
    fn test_derive_sensible_identifier_from_path() {
        assert_eq!(
            derive_sensible_identifier_from_path(&parse_quote! { Struct1 }).unwrap(),
            "Struct1"
        );
        assert_eq!(
            derive_sensible_identifier_from_path(&parse_quote! { std::test::Struct1 }).unwrap(),
            "Struct1"
        );
        assert_eq!(
            derive_sensible_identifier_from_path(&parse_quote! { Vec<u8> }).unwrap(),
            "Vec_u8"
        );
        assert_eq!(
            derive_sensible_identifier_from_path(&parse_quote! { Generic<'lifetime, u8> }).unwrap(),
            "Generic_lifetime_u8"
        );
        assert_eq!(
            derive_sensible_identifier_from_path(&parse_quote! { Generic::<'lifetime, u8> })
                .unwrap(),
            "Generic_lifetime_u8"
        );
        assert_eq!(
            derive_sensible_identifier_from_path(
                &parse_quote! { HashMap<String, std::rust::Test> }
            )
            .unwrap(),
            "HashMap_String_std_rust_Test"
        );
    }

    #[test]
    fn test_blueprint() {
        let input = TokenStream::from_str(
            "#[types(Struct1, Struct2 as Hi, u32, NonFungibleGlobalId, Vec<Hash>, Vec<Bucket> as GenericAlias)] mod test { struct Test {a: u32, admin: ResourceManager} impl Test { pub fn x(&self, i: u32) -> u32 { i + self.a } pub fn y(i: u32) -> u32 { i * 2 } } }",
        )
            .unwrap();
        let output = handle_blueprint(input).unwrap();

        assert_code_eq(
            output,
            quote! {

                #[automatically_derived]
                pub mod test {
                    use super::*;
                    use scrypto::prelude::*;
                    use scrypto::prelude::MethodAccessibility::*;
                    use scrypto::prelude::RoyaltyAmount::*;

                    fn method_auth_template() -> scrypto::blueprints::package::MethodAuthTemplate {
                        scrypto::blueprints::package::MethodAuthTemplate::AllowAll
                    }

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

                    impl HasMethods for Test {
                        type Permissions = Methods<MethodAccessibility>;
                        type Royalties = Methods<(RoyaltyAmount, bool)>;
                    }

                    impl HasTypeInfo for Test {
                        const PACKAGE_ADDRESS: Option<PackageAddress> = None;
                        const BLUEPRINT_NAME: &'static str = "Test";
                        const OWNED_TYPE_NAME: &'static str = "OwnedTest";
                        const GLOBAL_TYPE_NAME: &'static str = "GlobalTest";
                    }

                    pub struct Methods<T> {
                        x: T,
                    }

                    impl<T> MethodMapping<T> for Methods<T> {
                        const MODULE_ID: scrypto::api::ModuleId = scrypto::api::ModuleId::Main;

                        fn to_mapping(self) -> Vec<(String, T)> {
                            vec![
                                ("x".to_string(), self.x),
                            ]
                        }

                        fn methods() -> Vec<&'static str> {
                            vec![ "x", ]
                        }
                    }

                    pub struct Functions<T> {
                        y: T,
                    }

                    impl<T> FnMapping<T> for Functions<T> {
                        fn to_mapping(self) -> Vec<(String, T)> {
                            vec![
                                ("y".to_string(), self.y),
                            ]
                        }
                    }

                    pub struct Fns<T> {
                        x: T,
                        y: T,
                    }

                    impl<T> FnMapping<T> for Fns<T> {
                        fn to_mapping(self) -> Vec<(String, T)> {
                            vec![
                                ("x".to_string(), self.x),
                                ("y".to_string(), self.y),
                            ]
                        }
                    }

                    #[allow(non_camel_case_types)]
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct Test_x_Input { i : u32 }

                    #[allow(non_camel_case_types)]
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct Test_y_Input { i : u32 }

                    #[no_mangle]
                    pub extern "C" fn Test_x(args: ::scrypto::engine::wasm_api::Buffer) -> ::scrypto::engine::wasm_api::Slice {
                        use sbor::rust::ops::{Deref, DerefMut};

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
                        use sbor::rust::ops::{Deref, DerefMut};

                        // Set up panic hook
                        ::scrypto::set_up_panic_hook();

                        let input: Test_y_Input = ::scrypto::data::scrypto::scrypto_decode(&::scrypto::engine::wasm_api::copy_buffer(args)).unwrap();
                        let return_data = Test::y(input.i);
                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    fn function_auth() -> scrypto::blueprints::package::FunctionAuth {
                        scrypto::blueprints::package::FunctionAuth::AllowAll
                    }

                    fn package_royalty_config() -> PackageRoyaltyConfig {
                        PackageRoyaltyConfig::Disabled
                    }

                    #[no_mangle]
                    pub extern "C" fn Test_schema() -> ::scrypto::engine::wasm_api::Slice {
                        use ::scrypto::radix_blueprint_schema_init::*;
                        use sbor::rust::prelude::*;
                        use sbor::schema::*;
                        use sbor::*;

                        let schema = {
                            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
                            let mut fields = Vec::new();
                            let type_index = aggregator.add_child_type_and_descendents::<Test>();
                            fields.push(FieldSchema::static_field(type_index));

                            let state = BlueprintStateSchemaInit {
                                fields,
                                collections: Vec::new(),
                            };

                            let functions = {
                                let mut functions: IndexMap<String, FunctionSchemaInit> = index_map_new();
                                functions.insert(
                                    "x".to_string(),
                                    FunctionSchemaInit {
                                        receiver: Option::Some(::scrypto::radix_blueprint_schema_init::ReceiverInfo::normal_ref()),
                                        input: TypeRef::Static(aggregator.add_child_type_and_descendents::<Test_x_Input>()),
                                        output: TypeRef::Static(aggregator.add_child_type_and_descendents::<u32>()),
                                        export: "Test_x".to_string(),
                                    }
                                );
                                functions.insert(
                                    "y".to_string(),
                                    FunctionSchemaInit {
                                        receiver: Option::None,
                                        input: TypeRef::Static(aggregator.add_child_type_and_descendents::<Test_y_Input>()),
                                        output: TypeRef::Static(aggregator.add_child_type_and_descendents::<u32>()),
                                        export: "Test_y".to_string(),
                                    }
                                );

                                BlueprintFunctionsSchemaInit {
                                    functions,
                                }
                            };

                            let events = {
                                let mut event_schema = index_map_new();
                                BlueprintEventSchemaInit {
                                    event_schema,
                                }
                            };

                            let types = {
                                let mut type_schema = index_map_new();
                                {
                                    let local_type_index =
                                        aggregator.add_child_type_and_descendents::<Vec<Bucket> >();
                                    type_schema.insert("GenericAlias".to_owned(), local_type_index);
                                }
                                {
                                    let local_type_index = aggregator.add_child_type_and_descendents::<Struct2>();
                                    type_schema.insert("Hi".to_owned(), local_type_index);
                                }
                                {
                                    let local_type_index =
                                        aggregator.add_child_type_and_descendents::<NonFungibleGlobalId>();
                                    type_schema.insert("NonFungibleGlobalId".to_owned(), local_type_index);
                                }
                                {
                                    let local_type_index = aggregator.add_child_type_and_descendents::<Struct1>();
                                    type_schema.insert("Struct1".to_owned(), local_type_index);
                                }
                                {
                                    let local_type_index = aggregator.add_child_type_and_descendents::<Vec<Hash> >();
                                    type_schema.insert("Vec_Hash".to_owned(), local_type_index);
                                }
                                {
                                    let local_type_index = aggregator.add_child_type_and_descendents::<u32>();
                                    type_schema.insert("u32".to_owned(), local_type_index);
                                }
                                BlueprintTypeSchemaInit { type_schema }
                            };

                            let schema = generate_full_schema(aggregator);

                            BlueprintSchemaInit {
                                generics: vec![],
                                schema,
                                state,
                                events,
                                types,
                                functions,
                                hooks: BlueprintHooksInit::default(),
                            }
                        };

                        let mut dependencies = index_set_new();

                        let auth_config = {
                            scrypto::blueprints::package::AuthConfig {
                                method_auth: method_auth_template(),
                                function_auth: function_auth(),
                            }
                        };

                        let royalty_config = package_royalty_config();

                        let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
                            blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
                            is_transient: false,
                            feature_set: IndexSet::default(),
                            dependencies,
                            schema,
                            auth_config,
                            royalty_config,
                        };

                        return ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap());
                    }

                    #[allow(non_camel_case_types)]
                    #[derive(Clone, Copy, ::scrypto::prelude::ScryptoSbor)]
                    pub struct TestObjectStub {
                        pub handle: ::scrypto::component::ObjectStubHandle,
                    }

                    impl ::scrypto::component::ObjectStub for TestObjectStub {
                        type AddressType = ComponentAddress;

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
                        fn y(i: u32) -> u32;
                    }

                    impl TestFunctions for ::scrypto::component::Blueprint<Test> {
                        fn y(i: u32) -> u32 {
                            Self::call_function_raw("y", scrypto_args!(i))
                        }
                    }

                    pub trait TestRegisteredType {
                        fn blueprint_type_identifier() -> BlueprintTypeIdentifier;
                    }

                    pub trait TestKeyValueStore {
                        fn new_with_registered_type() -> Self;
                    }

                    pub trait TestResourceBuilder {
                        fn new_string_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                StringNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        >;
                        fn new_integer_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                IntegerNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        >;
                        fn new_bytes_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                BytesNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        >;
                        fn new_ruid_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                RUIDNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        >;
                    }

                    #[derive(ScryptoSbor)]
                    #[sbor(transparent)]
                    pub struct Hi(Struct2);

                    #[derive(ScryptoSbor)]
                    #[sbor(transparent)]
                    pub struct GenericAlias(Vec<Bucket>);

                    impl<
                            K: ScryptoEncode + ScryptoDecode + ScryptoDescribe + TestRegisteredType,
                            V: ScryptoEncode + ScryptoDecode + ScryptoDescribe + TestRegisteredType,
                        > TestKeyValueStore for KeyValueStore<K, V>
                    {
                        fn new_with_registered_type() -> Self {
                            let store_schema = RemoteKeyValueStoreDataSchema {
                                key_type: K::blueprint_type_identifier(),
                                value_type: V::blueprint_type_identifier(),
                                allow_ownership: true,
                            };
                            Self {
                                id: Own(ScryptoVmV1Api::kv_store_new(SborFixedEnumVariant::<
                                    KV_STORE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteKeyValueStoreDataSchema,
                                > {
                                    fields: store_schema,
                                })),
                                key: PhantomData,
                                value: PhantomData,
                            }
                        }
                    }

                    impl TestResourceBuilder for ResourceBuilder {
                        fn new_string_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                StringNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        > {
                            InProgressResourceBuilder::new(
                                owner_role,
                                NonFungibleResourceType::new(SborFixedEnumVariant {
                                    fields: RemoteNonFungibleDataSchema::new(
                                        D::blueprint_type_identifier(),
                                        D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                                    ),
                                }),
                            )
                        }
                        fn new_integer_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                IntegerNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        > {
                            InProgressResourceBuilder::new(
                                owner_role,
                                NonFungibleResourceType::new(SborFixedEnumVariant {
                                    fields: RemoteNonFungibleDataSchema::new(
                                        D::blueprint_type_identifier(),
                                        D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                                    ),
                                }),
                            )
                        }
                        fn new_bytes_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                BytesNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        > {
                            InProgressResourceBuilder::new(
                                owner_role,
                                NonFungibleResourceType::new(SborFixedEnumVariant {
                                    fields: RemoteNonFungibleDataSchema::new(
                                        D::blueprint_type_identifier(),
                                        D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                                    ),
                                }),
                            )
                        }
                        fn new_ruid_non_fungible_with_registered_type<D: NonFungibleData + TestRegisteredType>(
                            owner_role: OwnerRole,
                        ) -> InProgressResourceBuilder<
                            NonFungibleResourceType<
                                RUIDNonFungibleLocalId,
                                D,
                                SborFixedEnumVariant<
                                    NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE,
                                    RemoteNonFungibleDataSchema
                                >
                            >
                        > {
                            InProgressResourceBuilder::new(
                                owner_role,
                                NonFungibleResourceType::new(SborFixedEnumVariant {
                                    fields: RemoteNonFungibleDataSchema::new(
                                        D::blueprint_type_identifier(),
                                        D::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect()
                                    ),
                                }),
                            )
                        }
                    }

                    impl TestRegisteredType for Struct1 {
                        fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                            BlueprintTypeIdentifier {
                                package_address: Runtime::package_address(),
                                blueprint_name: "Test".to_owned(),
                                type_name: "Struct1".to_owned(),
                            }
                        }
                    }

                    impl From<Struct2> for Hi {
                        fn from(value: Struct2) -> Self {
                            Self(value)
                        }
                    }
                    impl From<Hi> for Struct2 {
                        fn from(value: Hi) -> Self {
                            value.0
                        }
                    }
                    impl TestRegisteredType for Hi {
                        fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                            BlueprintTypeIdentifier {
                                package_address: Runtime::package_address(),
                                blueprint_name: "Test".to_owned(),
                                type_name: "Hi".to_owned(),
                            }
                        }
                    }

                    impl TestRegisteredType for u32 {
                        fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                            BlueprintTypeIdentifier {
                                package_address: Runtime::package_address(),
                                blueprint_name: "Test".to_owned(),
                                type_name: "u32".to_owned(),
                            }
                        }
                    }

                    impl TestRegisteredType for NonFungibleGlobalId {
                        fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                            BlueprintTypeIdentifier {
                                package_address: Runtime::package_address(),
                                blueprint_name: "Test".to_owned(),
                                type_name: "NonFungibleGlobalId".to_owned(),
                            }
                        }
                    }

                    impl TestRegisteredType for Vec<Hash> {
                        fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                            BlueprintTypeIdentifier {
                                package_address: Runtime::package_address(),
                                blueprint_name: "Test".to_owned(),
                                type_name: "Vec_Hash".to_owned(),
                            }
                        }
                    }

                    impl From<Vec<Bucket> > for GenericAlias {
                        fn from(value: Vec<Bucket>) -> Self {
                            Self(value)
                        }
                    }
                    impl From<GenericAlias> for Vec<Bucket> {
                        fn from(value: GenericAlias) -> Self {
                            value.0
                        }
                    }
                    impl TestRegisteredType for GenericAlias {
                        fn blueprint_type_identifier() -> BlueprintTypeIdentifier {
                            BlueprintTypeIdentifier {
                                package_address: Runtime::package_address(),
                                blueprint_name: "Test".to_owned(),
                                type_name: "GenericAlias".to_owned(),
                            }
                        }
                    }
                }

                #[automatically_derived]
                pub mod test_test {
                    use super::*;
                    use scrypto::prelude::*;
                    use scrypto::prelude::MethodAccessibility::*;
                    use scrypto::prelude::RoyaltyAmount::*;

                    #[derive(:: scrypto :: prelude :: ScryptoSbor)]
                    pub struct TestState {
                        pub a: u32,
                        pub admin: ResourceManager
                    }

                    #[derive(Debug, Clone, Copy)]
                    pub struct Test(pub NodeId);

                    impl<D: sbor::Decoder<::scrypto::prelude::ScryptoCustomValueKind>>
                        ::scrypto::prelude::Decode<::scrypto::prelude::ScryptoCustomValueKind, D> for Test
                    {
                        #[inline]
                        fn decode_body_with_value_kind(
                            decoder: &mut D,
                            value_kind: ::scrypto::prelude::ValueKind<::scrypto::prelude::ScryptoCustomValueKind,>,
                        ) -> Result<Self, ::scrypto::prelude::DecodeError> {
                            let node_id = match value_kind {
                                ValueKind::Custom(::scrypto::prelude::ScryptoCustomValueKind::Reference) => {
                                    <::scrypto::prelude::Reference as ::scrypto::prelude::Decode<
                                        ::scrypto::prelude::ScryptoCustomValueKind,
                                        D,
                                    >>::decode_body_with_value_kind(decoder, value_kind)
                                    .map(|reference| reference.0)
                                }
                                ValueKind::Custom(::scrypto::prelude::ScryptoCustomValueKind::Own) => {
                                    <::scrypto::prelude::Own as ::scrypto::prelude::Decode<
                                        ::scrypto::prelude::ScryptoCustomValueKind,
                                        D,
                                    >>::decode_body_with_value_kind(decoder, value_kind)
                                    .map(|own| own.0)
                                }
                                _ => Err(::scrypto::prelude::DecodeError::InvalidCustomValue),
                            }?;
                            Ok(Self(node_id))
                        }
                    }

                    impl ::core::convert::TryFrom<Test> for ::scrypto::prelude::ComponentAddress {
                        type Error = ::scrypto::prelude::ParseComponentAddressError;
                        fn try_from(value: Test) -> Result<Self, Self::Error> {
                            ::scrypto::prelude::ComponentAddress::try_from(value.0)
                        }
                    }
                    impl ::core::convert::TryFrom<Test> for ::scrypto::prelude::ResourceAddress {
                        type Error = ::scrypto::prelude::ParseResourceAddressError;
                        fn try_from(value: Test) -> Result<Self, Self::Error> {
                            ::scrypto::prelude::ResourceAddress::try_from(value.0)
                        }
                    }
                    impl ::core::convert::TryFrom<Test> for ::scrypto::prelude::PackageAddress {
                        type Error = ::scrypto::prelude::ParsePackageAddressError;
                        fn try_from(value: Test) -> Result<Self, Self::Error> {
                            ::scrypto::prelude::PackageAddress::try_from(value.0)
                        }
                    }
                    impl ::core::convert::TryFrom<Test> for ::scrypto::prelude::GlobalAddress {
                        type Error = ::scrypto::prelude::ParseGlobalAddressError;
                        fn try_from(value: Test) -> Result<Self, Self::Error> {
                            ::scrypto::prelude::GlobalAddress::try_from(value.0)
                        }
                    }
                    impl ::core::convert::TryFrom<Test> for ::scrypto::prelude::InternalAddress {
                        type Error = ::scrypto::prelude::ParseInternalAddressError;
                        fn try_from(value: Test) -> Result<Self, Self::Error> {
                            ::scrypto::prelude::InternalAddress::try_from(value.0)
                        }
                    }
                    impl ::core::convert::From<Test> for ::scrypto::prelude::Own {
                        fn from(value: Test) -> Self {
                            Self(value.0)
                        }
                    }
                    impl ::core::convert::From<Test> for ::scrypto::prelude::Reference {
                        fn from(value: Test) -> Self {
                            Self(value.0)
                        }
                    }
                    impl ::core::convert::From<Test> for ::scrypto::prelude::NodeId {
                        fn from(value: Test) -> ::scrypto::prelude::NodeId {
                            value.0
                        }
                    }

                    impl Test {
                        pub fn x<Y: SystemApi<E>, E: SystemApiError>(&self, i: u32, env: &mut Y) -> Result<u32, E> {
                            let rtn = env.call_method(
                                &self.0,
                                stringify!(x),
                                ::scrypto::prelude::scrypto_encode(&(i,)).unwrap()
                            )?;
                            Ok(::scrypto::prelude::scrypto_decode(&rtn).unwrap())
                        }

                        pub fn y<Y: SystemApi<E>, E: SystemApiError>(
                            i: u32,
                            blueprint_package_address: ::scrypto::prelude::PackageAddress,
                            env: &mut Y
                        ) -> Result<u32, E> {
                            let rtn = env.call_function(
                                blueprint_package_address,
                                "Test",
                                stringify!(y),
                                ::scrypto::prelude::scrypto_encode(&(i,)).unwrap()
                            )?;
                            Ok(::scrypto::prelude::scrypto_decode(&rtn).unwrap())
                        }
                    }
                }
            },
        );
    }
}
