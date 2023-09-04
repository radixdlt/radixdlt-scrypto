//! This module converts the models from `schema.rs` to the `ast.rs` models which are eventually
//! converted to a TokenStream.

use heck::ToPascalCase;
use radix_engine_interface::prelude::*;

use crate::scrypto_bindgen::ast;
use crate::scrypto_bindgen::schema;

macro_rules! ident {
    ($ident: expr) => {
        syn::Ident::new($ident, proc_macro2::Span::call_site())
    };
}

pub fn blueprint_schema_stub_to_ast_stub<S>(
    schema_stub: schema::BlueprintStub,
    schema_resolver: &S,
) -> Result<ast::BlueprintStub, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    Ok(ast::BlueprintStub {
        invocation_fn_signature_items: schema_stub
            .functions
            .into_iter()
            .map(|func| {
                function_schema_stub_to_ast_stub(
                    func,
                    schema_stub.blueprint_name.clone(),
                    schema_resolver,
                )
            })
            .collect::<Result<_, _>>()?,
        blueprint_name: schema_stub.blueprint_name,
    })
}

pub fn function_schema_stub_to_ast_stub<S>(
    schema_stub: schema::Function,
    blueprint_name: String,
    schema_resolver: &S,
) -> Result<ast::InvocationFn, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    let fn_type = match schema_stub.receiver {
        Some(ReceiverInfo {
            ref_types: RefTypes::NORMAL | RefTypes::DIRECT_ACCESS,
            receiver: Receiver::SelfRef,
        }) => ast::FnType::Method {
            is_mutable_receiver: false,
        },
        Some(ReceiverInfo {
            ref_types: RefTypes::NORMAL | RefTypes::DIRECT_ACCESS,
            receiver: Receiver::SelfRefMut,
        }) => ast::FnType::Method {
            is_mutable_receiver: true,
        },
        None => ast::FnType::Function,
        _ => panic!("Invalid BitFlags for RefTypes"),
    };
    let ident = ident!(&schema_stub.ident);

    let inputs = schema_stub
        .arguments
        .into_iter()
        .map(|(arg_name, arg_type_index)| {
            type_name(&arg_type_index, blueprint_name.as_str(), schema_resolver).map(|type_name| {
                (
                    ident!(&arg_name),
                    syn::parse2(type_name.parse().unwrap()).unwrap(),
                )
            })
        })
        .collect::<Result<_, _>>()?;
    let output = syn::parse2(
        type_name(
            &schema_stub.returns,
            blueprint_name.as_str(),
            schema_resolver,
        )?
        .parse()
        .unwrap(),
    )
    .unwrap();

    Ok(ast::InvocationFn {
        inputs,
        fn_type,
        ident,
        output,
    })
}

fn type_name<S>(
    type_identifier: &TypeIdentifier,
    blueprint_name: &str,
    schema_resolver: &S,
) -> Result<String, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    let type_kind = schema_resolver.resolve_type_kind(type_identifier)?;
    let type_metadata = schema_resolver.resolve_type_metadata(type_identifier)?;
    let type_validation = schema_resolver.resolve_type_validation(type_identifier)?;

    let name = match type_kind {
        TypeKind::Any => "ScryptoValue".to_owned(),
        TypeKind::Bool => "bool".to_owned(),
        TypeKind::I8 => "i8".to_owned(),
        TypeKind::I16 => "i16".to_owned(),
        TypeKind::I32 => "i32".to_owned(),
        TypeKind::I64 => "i64".to_owned(),
        TypeKind::I128 => "i128".to_owned(),
        TypeKind::U8 => "u8".to_owned(),
        TypeKind::U16 => "u16".to_owned(),
        TypeKind::U32 => "u32".to_owned(),
        TypeKind::U64 => "u64".to_owned(),
        TypeKind::U128 => "u128".to_owned(),
        TypeKind::String => "String".to_owned(),
        TypeKind::Array { element_type } => {
            format!(
                "Vec<{}>",
                type_name(
                    &TypeIdentifier(type_identifier.0, element_type),
                    blueprint_name,
                    schema_resolver
                )?
            )
        }
        TypeKind::Tuple { field_types } => match type_metadata.get_name_string() {
            Some(name) => name,
            None => {
                if field_types.is_empty() {
                    "()".to_owned()
                } else {
                    format!(
                        "({},)",
                        field_types
                            .iter()
                            .map(|local_type_index| type_name(
                                &TypeIdentifier(type_identifier.0, *local_type_index),
                                blueprint_name,
                                schema_resolver
                            ))
                            .collect::<Result<Vec<String>, _>>()?
                            .join(", ")
                    )
                }
            }
        },
        TypeKind::Enum { variants } => {
            // There is currently no way to know if this type has generics or not. Thus, we need to
            // deal with generic enums from the standard library in a special way. We determine if
            // the type at hand is `Option` if: the name in the metadata is "Option" and the local
            // type index is of a well known type.
            match (
                type_metadata.get_name(),
                variants.len(),
                variants.get(&0).as_ref().map(|vec| vec.as_slice()),
                variants.get(&1).as_ref().map(|vec| vec.as_slice()),
            ) {
                (Some("Option"), 2usize, Some([]), Some([some_type_index])) => Ok(format!(
                    "Option<{}>",
                    type_name(
                        &TypeIdentifier(type_identifier.0, *some_type_index),
                        blueprint_name,
                        schema_resolver
                    )?
                )),
                (Some("Result"), 2usize, Some([ok_type_index]), Some([err_type_index])) => {
                    Ok(format!(
                        "Result<{}, {}>",
                        type_name(
                            &TypeIdentifier(type_identifier.0, *ok_type_index),
                            blueprint_name,
                            schema_resolver
                        )?,
                        type_name(
                            &TypeIdentifier(type_identifier.0, *err_type_index),
                            blueprint_name,
                            schema_resolver
                        )?
                    ))
                }
                (Some(name), ..) => Ok(name.to_owned()),
                (None, ..) => Err(schema::SchemaError::NoNameFound),
            }?
        }
        TypeKind::Map {
            key_type,
            value_type,
        } => format!(
            "BTreeMap<{}, {}>",
            type_name(
                &TypeIdentifier(type_identifier.0, key_type),
                blueprint_name,
                schema_resolver
            )?,
            type_name(
                &TypeIdentifier(type_identifier.0, value_type),
                blueprint_name,
                schema_resolver
            )?
        ),
        TypeKind::Custom(ScryptoCustomTypeKind::Decimal) => "Decimal".to_owned(),
        TypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal) => "PreciseDecimal".to_owned(),
        TypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId) => {
            "NonFungibleLocalId".to_owned()
        }

        TypeKind::Custom(ScryptoCustomTypeKind::Own)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsBucket,
                )) =>
        {
            "Bucket".to_owned()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsProof,
                )) =>
        {
            "Proof".to_owned()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsVault,
                )) =>
        {
            "Vault".to_owned()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsKeyValueStore,
                )) =>
        {
            "NodeId".to_owned()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsGlobalAddressReservation,
                )) =>
        {
            "GlobalAddressReservation".to_owned()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(None, blueprint_name.to_owned()),
                ))
                || type_validation
                    == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                        OwnValidation::IsTypedObject(
                            Some(schema_resolver.package_address()),
                            blueprint_name.to_owned(),
                        ),
                    )) =>
        {
            format!("Owned<{}>", blueprint_name.to_pascal_case())
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own) => "Own".to_owned(),

        TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobal,
                )) =>
        {
            "GlobalAddress".to_string()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalPackage,
                )) =>
        {
            "PackageAddress".to_string()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalComponent,
                )) =>
        {
            "ComponentAddress".to_string()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalResourceManager,
                )) =>
        {
            "ResourceAddress".to_string()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsInternal,
                )) =>
        {
            "InternalAddress".to_string()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalTyped(None, blueprint_name.to_owned()),
                ))
                || type_validation
                    == TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                        ReferenceValidation::IsGlobalTyped(
                            Some(schema_resolver.package_address()),
                            blueprint_name.to_owned(),
                        ),
                    )) =>
        {
            format!("Global<{}>", blueprint_name.to_pascal_case())
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Reference) => "Reference".to_owned(),
    };
    Ok(name)
}
