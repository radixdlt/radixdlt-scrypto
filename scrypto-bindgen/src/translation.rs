//! This module converts the models from `schema.rs` to the `ast.rs` models which are eventually
//! converted to a TokenStream.

use super::types::{BlueprintFunctionSignaturesReplacementMap, FunctionSignaturesReplacementMap};
use super::{ast, schema};
use crate::{ident, token_stream_from_str};
use radix_blueprint_schema_init::*;
use radix_common::prelude::*;

/// A list of the packages that we generate bindings for. This must match the list in the
/// update-bindings.sh script.
const PACKAGES_WITH_BINDINGS: &'static [PackageAddress] = &[
    FAUCET_PACKAGE,
    CONSENSUS_MANAGER_PACKAGE,
    IDENTITY_PACKAGE,
    ACCOUNT_PACKAGE,
    POOL_PACKAGE,
    ACCESS_CONTROLLER_PACKAGE,
    LOCKER_PACKAGE,
];

pub fn package_schema_interface_to_ast_interface<S>(
    schema_interface: schema::PackageInterface,
    package_address: PackageAddress,
    schema_resolver: &S,
    blueprint_replacement_map: &BlueprintFunctionSignaturesReplacementMap,
) -> Result<ast::PackageStub, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    Ok(ast::PackageStub {
        blueprints: schema_interface
            .blueprints
            .into_iter()
            .map(|(blueprint_name, blueprint_interface)| {
                blueprint_schema_interface_to_ast_interface(
                    blueprint_interface,
                    package_address,
                    blueprint_name,
                    schema_resolver,
                    blueprint_replacement_map,
                )
            })
            .collect::<Result<_, _>>()?,
        auxiliary_types: schema_auxiliary_types_to_ast_types(
            schema_interface.auxiliary_types,
            schema_resolver,
        )?,
    })
}

pub fn blueprint_schema_interface_to_ast_interface<S>(
    schema_interface: schema::BlueprintInterface,
    package_address: PackageAddress,
    blueprint_name: String,
    schema_resolver: &S,
    blueprint_replacement_map: &BlueprintFunctionSignaturesReplacementMap,
) -> Result<ast::BlueprintStub, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    Ok(ast::BlueprintStub {
        fn_signatures: schema_interface
            .functions
            .into_iter()
            .map(|func| {
                function_schema_interface_to_ast_interface(
                    func,
                    schema_resolver,
                    blueprint_replacement_map.get(&blueprint_name),
                )
            })
            .collect::<Result<_, _>>()?,
        blueprint_name,
        package_address,
    })
}

pub fn function_schema_interface_to_ast_interface<S>(
    schema_interface: schema::Function,
    schema_resolver: &S,
    func_sig_replacements_map: Option<&FunctionSignaturesReplacementMap>,
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
    let function_ident = ident!(&schema_interface.ident);

    // Check if there are some replacements for particular method name
    let func_sig_replacements = func_sig_replacements_map
        .and_then(|replacements_map| replacements_map.get(&schema_interface.ident));

    let inputs = schema_interface
        .arguments
        .into_iter()
        .enumerate()
        .map(|(idx, (arg_name, arg_type_index))| {
            // Get type replacement if exists for argument idx
            let ty = func_sig_replacements.and_then(|func| func.arg.get(&idx));

            let ty_name = match ty {
                Some(ty) => ty.clone(),
                None => type_name(&arg_type_index, schema_resolver)?,
            };
            Ok((ident!(&arg_name), token_stream_from_str!(&ty_name)))
        })
        .collect::<Result<_, _>>()?;

    // Get type replacement if exists for return type
    let ty = func_sig_replacements.and_then(|func| func.output.clone());
    let ty_name = match ty {
        Some(ty) => ty,
        None => type_name(&schema_interface.returns, schema_resolver)?,
    };
    let output = token_stream_from_str!(&ty_name);

    Ok(ast::FnSignature {
        inputs,
        fn_type,
        ident: function_ident,
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
    let metadata_type_name = type_metadata.get_name_string();

    let name = match type_kind {
        TypeKind::Any => metadata_type_name.unwrap_or("ScryptoValue".to_owned()),
        TypeKind::Bool => metadata_type_name.unwrap_or("bool".to_owned()),
        TypeKind::I8 => metadata_type_name.unwrap_or("i8".to_owned()),
        TypeKind::I16 => metadata_type_name.unwrap_or("i16".to_owned()),
        TypeKind::I32 => metadata_type_name.unwrap_or("i32".to_owned()),
        TypeKind::I64 => metadata_type_name.unwrap_or("i64".to_owned()),
        TypeKind::I128 => metadata_type_name.unwrap_or("i128".to_owned()),
        TypeKind::U8 => metadata_type_name.unwrap_or("u8".to_owned()),
        TypeKind::U16 => metadata_type_name.unwrap_or("u16".to_owned()),
        TypeKind::U32 => metadata_type_name.unwrap_or("u32".to_owned()),
        TypeKind::U64 => metadata_type_name.unwrap_or("u64".to_owned()),
        TypeKind::U128 => metadata_type_name.unwrap_or("u128".to_owned()),
        TypeKind::String => metadata_type_name.unwrap_or("String".to_owned()),
        TypeKind::Array { element_type } => metadata_type_name.unwrap_or(format!(
            "Vec<{}>",
            type_name(
                &ScopedTypeId(type_identifier.0, element_type),
                schema_resolver,
            )?
        )),
        TypeKind::Tuple { field_types } => {
            metadata_type_name.unwrap_or(match field_types.as_slice() {
                [] => "()".to_owned(),
                types => format!(
                    "({},)",
                    types
                        .iter()
                        .map(|local_type_index| type_name(
                            &ScopedTypeId(type_identifier.0, *local_type_index),
                            schema_resolver,
                        ))
                        .collect::<Result<Vec<String>, _>>()?
                        .join(", ")
                ),
            })
        }
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
                        schema_resolver,
                    )?
                )),
                (Some("Result"), 2usize, Some([ok_type_index]), Some([err_type_index])) => {
                    Ok(format!(
                        "Result<{}, {}>",
                        type_name(
                            &ScopedTypeId(type_identifier.0, *ok_type_index),
                            schema_resolver,
                        )?,
                        type_name(
                            &ScopedTypeId(type_identifier.0, *err_type_index),
                            schema_resolver,
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
        } => metadata_type_name.unwrap_or(format!(
            "IndexMap<{}, {}>",
            type_name(&ScopedTypeId(type_identifier.0, key_type), schema_resolver,)?,
            type_name(
                &ScopedTypeId(type_identifier.0, value_type),
                schema_resolver,
            )?
        )),
        TypeKind::Custom(custom_type_kind) => match custom_type_kind {
            ScryptoCustomTypeKind::Reference => match type_validation {
                TypeValidation::None => metadata_type_name.unwrap_or("Reference".to_owned()),
                TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    reference_type_validation,
                )) => match reference_type_validation {
                    ReferenceValidation::IsGlobal => "GlobalAddress".to_owned(),
                    ReferenceValidation::IsGlobalPackage => "PackageAddress".to_owned(),
                    ReferenceValidation::IsGlobalComponent => "ComponentAddress".to_owned(),
                    ReferenceValidation::IsGlobalResourceManager => "ResourceAddress".to_owned(),
                    ReferenceValidation::IsGlobalTyped(package_address, blueprint_name) => {
                        let this_package_address = schema_resolver.package_address();
                        if package_address.is_none()
                            || PACKAGES_WITH_BINDINGS.contains(&this_package_address)
                            || package_address.is_some_and(|package_address| {
                                this_package_address == package_address
                            })
                        {
                            format!("Global<{}>", blueprint_name)
                        } else {
                            metadata_type_name.unwrap_or("Reference".to_owned())
                        }
                    }
                    ReferenceValidation::IsInternal
                    | ReferenceValidation::IsInternalTyped(_, _) => "InternalAddress".to_owned(),
                },
                TypeValidation::I8(_)
                | TypeValidation::I16(_)
                | TypeValidation::I32(_)
                | TypeValidation::I64(_)
                | TypeValidation::I128(_)
                | TypeValidation::U8(_)
                | TypeValidation::U16(_)
                | TypeValidation::U32(_)
                | TypeValidation::U64(_)
                | TypeValidation::U128(_)
                | TypeValidation::String(_)
                | TypeValidation::Array(_)
                | TypeValidation::Map(_)
                | TypeValidation::Custom(ScryptoCustomTypeValidation::Own(..)) => {
                    panic!("Unexpected state: a reference type with non-reference validation.")
                }
            },
            ScryptoCustomTypeKind::Own => match type_validation {
                TypeValidation::None => metadata_type_name.unwrap_or("Own".to_owned()),
                TypeValidation::Custom(ScryptoCustomTypeValidation::Own(own_type_validation)) => {
                    match own_type_validation {
                        OwnValidation::IsBucket => {
                            metadata_type_name.unwrap_or("Bucket".to_owned())
                        }
                        OwnValidation::IsProof => metadata_type_name.unwrap_or("Proof".to_owned()),
                        OwnValidation::IsVault => metadata_type_name.unwrap_or("Vault".to_owned()),
                        OwnValidation::IsKeyValueStore => "Own".to_owned(),
                        OwnValidation::IsGlobalAddressReservation => {
                            metadata_type_name.unwrap_or("GlobalAddressReservation".to_owned())
                        }
                        OwnValidation::IsTypedObject(package_address, blueprint_name) => {
                            let this_package_address = schema_resolver.package_address();
                            if package_address.is_none()
                                || PACKAGES_WITH_BINDINGS.contains(&this_package_address)
                                || package_address.is_some_and(|package_address| {
                                    this_package_address == package_address
                                })
                            {
                                format!("Own<{}>", blueprint_name)
                            } else {
                                metadata_type_name.unwrap_or("Own".to_owned())
                            }
                        }
                    }
                }
                TypeValidation::I8(_)
                | TypeValidation::I16(_)
                | TypeValidation::I32(_)
                | TypeValidation::I64(_)
                | TypeValidation::I128(_)
                | TypeValidation::U8(_)
                | TypeValidation::U16(_)
                | TypeValidation::U32(_)
                | TypeValidation::U64(_)
                | TypeValidation::U128(_)
                | TypeValidation::String(_)
                | TypeValidation::Array(_)
                | TypeValidation::Map(_)
                | TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(..)) => {
                    panic!("Unexpected state: an own type with non-own validation.")
                }
            },
            ScryptoCustomTypeKind::Decimal => metadata_type_name.unwrap_or("Decimal".to_owned()),
            ScryptoCustomTypeKind::PreciseDecimal => {
                metadata_type_name.unwrap_or("PreciseDecimal".to_owned())
            }
            ScryptoCustomTypeKind::NonFungibleLocalId => {
                metadata_type_name.unwrap_or("NonFungibleLocalId".to_owned())
            }
        },
    };

    Ok(name)
}

pub fn schema_auxiliary_types_to_ast_types<S>(
    auxiliary_types: HashSet<ScopedTypeId>,
    schema_resolver: &S,
) -> Result<Vec<ast::AuxiliaryType>, schema::SchemaError>
where
    S: schema::PackageSchemaResolver,
{
    let mut ast_auxiliary_types = Vec::default();

    // No auxiliary types need to be generated if they're already well-known (i.e., they can just be
    // imported through scrypto::prelude::*). Thus, well known types are ignored.
    for scoped_type_id in auxiliary_types
        .into_iter()
        .filter_map(|item| match item.1 {
            LocalTypeId::SchemaLocalIndex(local_index) => Some((item.0, local_index)),
            LocalTypeId::WellKnown(..) => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|(schema_hash, local_type_index)| {
            ScopedTypeId(schema_hash, LocalTypeId::SchemaLocalIndex(local_type_index))
        })
    {
        let type_kind = schema_resolver.resolve_type_kind(&scoped_type_id)?;
        let type_metadata = schema_resolver.resolve_type_metadata(&scoped_type_id)?;

        let ast_auxiliary_type = match type_kind {
            TypeKind::Tuple { field_types } => {
                let Some(ref struct_name) = type_metadata.type_name else {
                    /* No type name = just a tuple. No generation needed. */
                    continue;
                };

                match type_metadata.get_field_names() {
                    /* Named fields struct */
                    Some(field_names) => ast::AuxiliaryType::NamedFieldsStruct {
                        struct_name: struct_name.as_ref().into(),
                        fields: field_names
                            .iter()
                            .zip(field_types)
                            .map(|(field_name, field_type)| {
                                let field_type_name = type_name(
                                    &ScopedTypeId(scoped_type_id.0, field_type),
                                    schema_resolver,
                                )?;

                                Ok((field_name.as_ref().into(), field_type_name))
                            })
                            .collect::<Result<_, _>>()?,
                    },
                    /* Tuple struct */
                    None => ast::AuxiliaryType::TupleStruct {
                        struct_name: struct_name.as_ref().into(),
                        field_types: field_types
                            .iter()
                            .map(|field_type| {
                                type_name(
                                    &ScopedTypeId(scoped_type_id.0, *field_type),
                                    schema_resolver,
                                )
                            })
                            .collect::<Result<_, _>>()?,
                    },
                }
            }
            TypeKind::Enum { variants } => {
                let enum_name = type_metadata
                    .type_name
                    .expect("Unexpected state: encountered an enum that has no type name!");

                // TODO: Determine a better way to not generate "Option" and "Result" as auxiliary
                // types that is not just based on the name.
                if enum_name == "Option" || enum_name == "Result" {
                    continue;
                }

                let Some(ChildNames::EnumVariants(variant_metadata)) = type_metadata.child_names
                else {
                    panic!("Unexpected state: the child names of an enum must be enum-variants")
                };

                let mut enum_variants = Vec::default();

                for variant_id in variants.keys() {
                    let field_types = variants
                        .get(variant_id)
                        .expect("Unexpected state: variant id can not be found!");
                    let metadata = variant_metadata
                        .get(variant_id)
                        .expect("Unexpected state: variant id can not be found!");

                    let variant_name = metadata
                        .type_name
                        .as_ref()
                        .expect("Unexpected state: an enum variant with no name!");

                    let enum_variant = if field_types.is_empty() {
                        ast::EnumVariant::Unit {
                            variant_name: variant_name.as_ref().into(),
                            variant_index: *variant_id,
                        }
                    } else {
                        match metadata.get_field_names() {
                            /* Named fields struct */
                            Some(field_names) => ast::EnumVariant::NamedFields {
                                variant_name: variant_name.as_ref().into(),
                                variant_index: *variant_id,
                                fields: field_names
                                    .iter()
                                    .zip(field_types)
                                    .map(|(field_name, field_type)| {
                                        let field_type_name = type_name(
                                            &ScopedTypeId(scoped_type_id.0, *field_type),
                                            schema_resolver,
                                        )?;

                                        Ok((field_name.as_ref().into(), field_type_name))
                                    })
                                    .collect::<Result<_, _>>()?,
                            },
                            /* Tuple struct */
                            None => ast::EnumVariant::Tuple {
                                variant_name: variant_name.as_ref().into(),
                                variant_index: *variant_id,
                                field_types: field_types
                                    .iter()
                                    .map(|field_type| {
                                        type_name(
                                            &ScopedTypeId(scoped_type_id.0, *field_type),
                                            schema_resolver,
                                        )
                                    })
                                    .collect::<Result<_, _>>()?,
                            },
                        }
                    };
                    enum_variants.push(enum_variant)
                }

                ast::AuxiliaryType::Enum {
                    enum_name: enum_name.into(),
                    variants: enum_variants,
                }
            }
            TypeKind::Any
            | TypeKind::Bool
            | TypeKind::I8
            | TypeKind::I16
            | TypeKind::I32
            | TypeKind::I64
            | TypeKind::I128
            | TypeKind::U8
            | TypeKind::U16
            | TypeKind::U32
            | TypeKind::U64
            | TypeKind::U128
            | TypeKind::String
            | TypeKind::Array { .. }
            | TypeKind::Map { .. }
            | TypeKind::Custom(..) => {
                /* Not considered an auxiliary type - no generation needed */
                continue;
            }
        };

        ast_auxiliary_types.push(ast_auxiliary_type)
    }

    ast_auxiliary_types.dedup();

    Ok(ast_auxiliary_types)
}
