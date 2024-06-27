use radix_blueprint_schema_init::*;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::package::*;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};

pub trait PackageSchemaResolver {
    fn lookup_schema(&self, schema_hash: &SchemaHash) -> Option<Rc<VersionedScryptoSchema>>;

    fn resolve_type_kind(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<LocalTypeKind<ScryptoCustomSchema>, SchemaError>;

    fn resolve_type_metadata(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<TypeMetadata, SchemaError>;

    fn resolve_type_validation(
        &self,
        type_identifier: &ScopedTypeId,
    ) -> Result<TypeValidation<ScryptoCustomTypeValidation>, SchemaError>;

    fn package_address(&self) -> PackageAddress;
}

pub fn package_interface_from_package_definition<S>(
    package_definition: BTreeMap<BlueprintVersionKey, BlueprintDefinition>,
    schema_resolver: &S,
) -> Result<PackageInterface, SchemaError>
where
    S: PackageSchemaResolver,
{
    let mut package_interface = PackageInterface::default();

    for (blueprint_key, blueprint_definition) in package_definition.into_iter() {
        let blueprint_name = blueprint_key.blueprint;

        if let Some((_, fields)) = blueprint_definition.interface.state.fields {
            for field in fields {
                if let BlueprintPayloadDef::Static(scoped_type_id) = field.field {
                    package_interface.auxiliary_types.insert(scoped_type_id);
                }
            }
        }

        let functions = &mut package_interface
            .blueprints
            .entry(blueprint_name)
            .or_default()
            .functions;

        for (function_name, function_schema) in blueprint_definition.interface.functions {
            let BlueprintPayloadDef::Static(input_type_identifier) = &function_schema.input else {
                Err(SchemaError::GenericTypeRefsNotSupported)?
            };

            // Input Types
            let inputs_scoped_type_id = {
                let type_kind = schema_resolver.resolve_type_kind(input_type_identifier)?;
                if let TypeKind::Tuple { field_types } = type_kind {
                    Ok(field_types
                        .into_iter()
                        .map(|local_type_id| ScopedTypeId(input_type_identifier.0, local_type_id))
                        .collect::<Vec<_>>())
                } else {
                    Err(SchemaError::FunctionInputIsNotATuple(
                        *input_type_identifier,
                    ))
                }
            }?;

            // Input Field Names
            let inputs_field_names = {
                let type_metadata = schema_resolver.resolve_type_metadata(input_type_identifier)?;
                match type_metadata.child_names.as_ref() {
                    /* Encountered a struct with field names, return them. */
                    Some(ChildNames::NamedFields(field_names)) => field_names
                        .iter()
                        .map(|entry| entry.as_ref().to_owned())
                        .collect::<Vec<_>>(),
                    /* A struct that has enum variants?? */
                    Some(ChildNames::EnumVariants(..)) => {
                        panic!(
                            "We have checked that this is a Tuple and it can't have enum variants."
                        )
                    }
                    /* Encountered a tuple-struct. Generate field names as `arg{n}` */
                    None => (0..inputs_scoped_type_id.len())
                        .map(|i| format!("arg{i}"))
                        .collect::<Vec<_>>(),
                }
            };

            // Output types
            let BlueprintPayloadDef::Static(output_local_type_index) = &function_schema.output
            else {
                return Err(SchemaError::GenericTypeRefsNotSupported);
            };

            // Auxiliary types
            // The auxiliary types are found by walking the input and output of each function and
            // storing the type-ids encountered in the inputs and outputs.
            for input_type in inputs_scoped_type_id.iter() {
                get_scoped_type_ids_in_path(
                    input_type,
                    schema_resolver,
                    &mut package_interface.auxiliary_types,
                )?;
            }
            get_scoped_type_ids_in_path(
                output_local_type_index,
                schema_resolver,
                &mut package_interface.auxiliary_types,
            )?;

            let function = Function {
                ident: function_name.to_owned(),
                receiver: function_schema.receiver.clone(),
                arguments: inputs_field_names
                    .into_iter()
                    .zip(inputs_scoped_type_id)
                    .collect::<IndexMap<String, ScopedTypeId>>(),
                returns: *output_local_type_index,
            };
            functions.push(function);
        }
    }

    Ok(package_interface)
}

#[derive(Clone, Debug, Default)]
pub struct PackageInterface {
    /// The interface definition of the various blueprints contained in the package. The key is the
    /// blueprint name and the value is the interface of the blueprint.
    pub blueprints: IndexMap<String, BlueprintInterface>,
    /// A set of [`ScopedTypeId`] of the auxiliary types found in the package interface. Auxiliary
    /// types are types which appear somewhere in the interface of the package. As an example, an
    /// enum that appears as a function input that requires generation for the interface to make
    /// sense.
    pub auxiliary_types: HashSet<ScopedTypeId>,
}

#[derive(Clone, Debug, Default)]
pub struct BlueprintInterface {
    /// The functions and methods encountered in the blueprint interface.
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub ident: String,
    pub receiver: Option<ReceiverInfo>,
    pub arguments: IndexMap<String, ScopedTypeId>,
    pub returns: ScopedTypeId,
}

fn get_scoped_type_ids_in_path<S>(
    type_id: &ScopedTypeId,
    schema_resolver: &S,
    collection: &mut HashSet<ScopedTypeId>,
) -> Result<(), SchemaError>
where
    S: PackageSchemaResolver,
{
    if !collection.insert(*type_id) {
        return Ok(());
    }

    let type_kind = schema_resolver.resolve_type_kind(type_id)?;

    match type_kind {
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
        | TypeKind::Custom(ScryptoCustomTypeKind::Reference)
        | TypeKind::Custom(ScryptoCustomTypeKind::Own)
        | TypeKind::Custom(ScryptoCustomTypeKind::Decimal)
        | TypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal)
        | TypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId) => {}
        TypeKind::Array { element_type } => {
            let scoped_type_id = ScopedTypeId(type_id.0, element_type);
            get_scoped_type_ids_in_path(&scoped_type_id, schema_resolver, collection)?;
        }
        TypeKind::Tuple { field_types } => {
            for field_type in field_types {
                let scoped_type_id = ScopedTypeId(type_id.0, field_type);
                get_scoped_type_ids_in_path(&scoped_type_id, schema_resolver, collection)?;
            }
        }
        TypeKind::Enum { variants } => {
            for field_types in variants.values() {
                for field_type in field_types {
                    let scoped_type_id = ScopedTypeId(type_id.0, *field_type);
                    get_scoped_type_ids_in_path(&scoped_type_id, schema_resolver, collection)?;
                }
            }
        }
        TypeKind::Map {
            key_type,
            value_type,
        } => {
            let scoped_type_id = ScopedTypeId(type_id.0, key_type);
            get_scoped_type_ids_in_path(&scoped_type_id, schema_resolver, collection)?;

            let scoped_type_id = ScopedTypeId(type_id.0, value_type);
            get_scoped_type_ids_in_path(&scoped_type_id, schema_resolver, collection)?;
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum SchemaError {
    FunctionInputIsNotATuple(ScopedTypeId),
    NonExistentLocalTypeIndex(LocalTypeId),
    FailedToGetSchemaFromSchemaHash,
    GenericTypeRefsNotSupported,
    NoNameFound,
}

impl Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
