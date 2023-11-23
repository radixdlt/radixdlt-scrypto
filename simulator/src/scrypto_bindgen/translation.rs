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

pub fn blueprint_schema_interface_to_ast_interface<S>(
    schema_interface: schema::BlueprintInterface,
    schema_resolver: &S,
) -> Result<ast::BlueprintStub, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    Ok(ast::BlueprintStub {
        fn_signatures: schema_interface
            .functions
            .into_iter()
            .map(|func| function_schema_interface_to_ast_interface(func, schema_resolver))
            .collect::<Result<_, _>>()?,
        blueprint_name: schema_interface.blueprint_name,
    })
}

pub fn function_schema_interface_to_ast_interface<S>(
    schema_interface: schema::Function,
    schema_resolver: &S,
) -> Result<ast::FnSignature, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    let fn_type = match schema_interface.receiver {
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
    let ident = ident!(&schema_interface.ident);

    let inputs = schema_interface
        .arguments
        .into_iter()
        .map(|(arg_name, arg_type_index)| {
            type_name(&arg_type_index, schema_resolver).map(|type_name| {
                (
                    ident!(&arg_name),
                    syn::parse2(type_name.parse().unwrap()).unwrap(),
                )
            })
        })
        .collect::<Result<_, _>>()?;
    let output = syn::parse2(
        type_name(&schema_interface.returns, schema_resolver)?
            .parse()
            .unwrap(),
    )
    .unwrap();

    Ok(ast::FnSignature {
        inputs,
        fn_type,
        ident,
        output,
    })
}

fn type_name<S>(
    type_identifier: &ScopedTypeId,
    schema_resolver: &S,
) -> Result<String, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    let type_kind = schema_resolver.resolve_type_kind(type_identifier)?;
    let type_metadata = schema_resolver.resolve_type_metadata(type_identifier)?;
    let type_validation = schema_resolver.resolve_type_validation(type_identifier)?;
    let type_ident = type_metadata.get_name_string();

    let name = match type_kind {
        TypeKind::Any => type_ident.unwrap_or("ScryptoValue".to_owned()),
        TypeKind::Bool => type_ident.unwrap_or("bool".to_owned()),
        TypeKind::I8 => type_ident.unwrap_or("i8".to_owned()),
        TypeKind::I16 => type_ident.unwrap_or("i16".to_owned()),
        TypeKind::I32 => type_ident.unwrap_or("i32".to_owned()),
        TypeKind::I64 => type_ident.unwrap_or("i64".to_owned()),
        TypeKind::I128 => type_ident.unwrap_or("i128".to_owned()),
        TypeKind::U8 => type_ident.unwrap_or("u8".to_owned()),
        TypeKind::U16 => type_ident.unwrap_or("u16".to_owned()),
        TypeKind::U32 => type_ident.unwrap_or("u32".to_owned()),
        TypeKind::U64 => type_ident.unwrap_or("u64".to_owned()),
        TypeKind::U128 => type_ident.unwrap_or("u128".to_owned()),
        TypeKind::String => type_ident.unwrap_or("String".to_owned()),
        TypeKind::Array { element_type } => type_ident.unwrap_or(format!(
            "Vec<{}>",
            type_name(
                &ScopedTypeId(type_identifier.0, element_type),
                schema_resolver
            )?
        )),
        TypeKind::Tuple { field_types } => type_ident.unwrap_or(if field_types.is_empty() {
            "()".to_owned()
        } else {
            format!(
                "({},)",
                field_types
                    .iter()
                    .map(|local_type_index| type_name(
                        &ScopedTypeId(type_identifier.0, *local_type_index),
                        schema_resolver
                    ))
                    .collect::<Result<Vec<String>, _>>()?
                    .join(", ")
            )
        }),
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
                        &ScopedTypeId(type_identifier.0, *some_type_index),
                        schema_resolver
                    )?
                )),
                (Some("Result"), 2usize, Some([ok_type_index]), Some([err_type_index])) => {
                    Ok(format!(
                        "Result<{}, {}>",
                        type_name(
                            &ScopedTypeId(type_identifier.0, *ok_type_index),
                            schema_resolver
                        )?,
                        type_name(
                            &ScopedTypeId(type_identifier.0, *err_type_index),
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
        } => type_ident.unwrap_or(format!(
            "IndexMap<{}, {}>",
            type_name(&ScopedTypeId(type_identifier.0, key_type), schema_resolver)?,
            type_name(
                &ScopedTypeId(type_identifier.0, value_type),
                schema_resolver
            )?
        )),
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
            "Own".to_owned()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own)
            if type_validation
                == TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsGlobalAddressReservation,
                )) =>
        {
            "GlobalAddressReservation".to_owned()
        }
        TypeKind::Custom(ScryptoCustomTypeKind::Own) => match type_validation {
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(package_address, bp_name),
            )) if package_address == Some(RESOURCE_PACKAGE)
                && bp_name == FUNGIBLE_BUCKET_BLUEPRINT =>
            {
                "FungibleBucket".to_owned()
            }
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(package_address, bp_name),
            )) if package_address == Some(RESOURCE_PACKAGE)
                && bp_name == NON_FUNGIBLE_BUCKET_BLUEPRINT =>
            {
                "NonFungibleBucket".to_owned()
            }
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(package_address, bp_name),
            )) if package_address.is_none()
                || package_address == Some(schema_resolver.package_address()) =>
            {
                format!("Owned<{}>", bp_name.to_pascal_case())
            }
            _ => "Own".to_owned(),
        },

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
        TypeKind::Custom(ScryptoCustomTypeKind::Reference) => match type_validation {
            TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalTyped(package_address, bp_name),
            )) if package_address.is_none()
                || package_address == Some(schema_resolver.package_address()) =>
            {
                format!("Global<{}>", bp_name.to_pascal_case())
            }
            _ => "Reference".to_owned(),
        },
    };
    Ok(name)
}
