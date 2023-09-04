use std::collections::BTreeMap;
use std::fmt::{Debug, Display};

use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::prelude::*;
use sbor::prelude::IndexMap;
use sbor::*;

pub trait PackageSchemaResolver {
    fn lookup_schema(&self, schema_hash: &SchemaHash) -> Option<VersionedScryptoSchema>;

    fn resolve_type_kind(
        &self,
        type_identifier: &TypeIdentifier,
    ) -> Result<SchemaTypeKind<ScryptoCustomSchema>, SchemaError>;

    fn resolve_type_metadata(
        &self,
        type_identifier: &TypeIdentifier,
    ) -> Result<TypeMetadata, SchemaError>;

    fn resolve_type_validation(
        &self,
        type_identifier: &TypeIdentifier,
    ) -> Result<TypeValidation<ScryptoCustomTypeValidation>, SchemaError>;

    fn package_address(&self) -> PackageAddress;
}

pub fn generate_blueprint_stubs<S>(
    package_definition: BTreeMap<BlueprintVersionKey, BlueprintDefinition>,
    schema_resolver: &S,
) -> Result<Vec<BlueprintStub>, SchemaError>
where
    S: PackageSchemaResolver,
{
    let mut blueprint_stubs = vec![];

    for (blueprint_key, blueprint_definition) in package_definition.into_iter() {
        let blueprint_ident = blueprint_key.blueprint;

        let mut functions = vec![];
        for (fn_ident, fn_schema) in blueprint_definition.interface.functions {
            let BlueprintPayloadDef::Static(args_type_identifier) = &fn_schema.input else {
                Err(SchemaError::GenericTypeRefsNotSupported)?
            };

            // Arg Names
            let arg_names = {
                let args_type_metadata =
                    schema_resolver.resolve_type_metadata(args_type_identifier)?;
                args_type_metadata
                    .child_names
                    .as_ref()
                    .map_or(Vec::new(), |names| match names {
                        ChildNames::NamedFields(named_fields) => named_fields
                            .iter()
                            .map(|v| v.clone().into_owned())
                            .collect(),
                        ChildNames::EnumVariants(..) => panic!("Impossible Case"),
                    })
            };
            let arg_type_indices = {
                let args_type_kind = schema_resolver.resolve_type_kind(args_type_identifier)?;
                if let TypeKind::Tuple { field_types } = args_type_kind {
                    Ok(field_types)
                } else {
                    Err(SchemaError::FunctionInputIsNotATuple(*args_type_identifier))
                }
            }?;
            assert_eq!(
                arg_names.len(),
                arg_type_indices.len(),
                "Arg names length != arg names type identifiers length"
            );

            let function = Function {
                ident: fn_ident.to_owned(),
                receiver: fn_schema.receiver.clone(),
                arguments: arg_names
                    .into_iter()
                    .zip(arg_type_indices.iter().map(|local_type_index| {
                        TypeIdentifier(args_type_identifier.0, *local_type_index)
                    }))
                    .collect::<IndexMap<String, TypeIdentifier>>(),
                returns: if let BlueprintPayloadDef::Static(output_local_type_index) =
                    &fn_schema.output
                {
                    *output_local_type_index
                } else {
                    Err(SchemaError::GenericTypeRefsNotSupported)?
                },
            };
            functions.push(function);
        }
        blueprint_stubs.push(BlueprintStub {
            functions,
            blueprint_name: blueprint_ident.to_owned(),
        })
    }

    Ok(blueprint_stubs)
}

pub struct BlueprintStub {
    pub blueprint_name: String,
    pub functions: Vec<Function>,
}

pub struct Function {
    pub ident: String,
    pub receiver: Option<ReceiverInfo>,
    pub arguments: IndexMap<String, TypeIdentifier>,
    pub returns: TypeIdentifier,
}

#[derive(Clone, Debug)]
pub enum SchemaError {
    FunctionInputIsNotATuple(TypeIdentifier),
    NonExistentLocalTypeIndex(LocalTypeIndex),
    SchemaValidationError(SchemaValidationError),
    FailedToGetSchemaFromSchemaHash,
    GenericTypeRefsNotSupported,
    NoNameFound,
}

impl Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
