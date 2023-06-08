use crate::blueprints::native_schema::*;
use crate::blueprints::package::*;
use crate::blueprints::pool::multi_resource_pool::*;
use crate::blueprints::pool::one_resource_pool::*;
use crate::blueprints::pool::two_resource_pool::*;
use radix_engine_common::data::manifest::*;
use radix_engine_common::prelude::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::BlueprintSchema;
use sbor::validate_payload_against_schema;
use sbor::LocalTypeIndex;
use sbor::Schema;
use transaction::prelude::InstructionV1;

pub fn validate_call_arguments_to_native_components(
    instructions: &[InstructionV1],
) -> Result<(), LocatedInstructionSchemaValidationError> {
    for (index, instruction) in instructions.iter().enumerate() {
        let (invocation, args) = match instruction {
            InstructionV1::CallFunction {
                package_address,
                blueprint_name,
                function_name,
                args,
            } => (
                Invocation::Function(
                    *package_address,
                    blueprint_name.to_owned(),
                    function_name.to_owned(),
                ),
                args,
            ),
            InstructionV1::CallMethod {
                address,
                method_name,
                args,
            } => (
                Invocation::Method(*address, ObjectModuleId::Main, method_name.to_owned()),
                args,
            ),
            InstructionV1::CallMetadataMethod {
                address,
                method_name,
                args,
            } => (
                Invocation::Method(*address, ObjectModuleId::Metadata, method_name.to_owned()),
                args,
            ),
            InstructionV1::CallRoyaltyMethod {
                address,
                method_name,
                args,
            } => (
                Invocation::Method(*address, ObjectModuleId::Royalty, method_name.to_owned()),
                args,
            ),
            InstructionV1::CallAccessRulesMethod {
                address,
                method_name,
                args,
            } => (
                Invocation::Method(
                    *address,
                    ObjectModuleId::AccessRules,
                    method_name.to_owned(),
                ),
                args,
            ),
            _ => continue,
        };

        let schema = get_arguments_schema(invocation).map_err(|cause| {
            LocatedInstructionSchemaValidationError {
                instruction_index: index,
                cause,
            }
        })?;
        if let Some((local_type_index, schema)) = schema {
            validate_payload_against_schema::<ManifestCustomExtension, _>(
                &manifest_encode(&args).unwrap(),
                schema,
                local_type_index,
                &(),
            )
            .map_err(|error| LocatedInstructionSchemaValidationError {
                instruction_index: index,
                cause: InstructionSchemaValidationError::SchemaValidationError(format!(
                    "{:?}",
                    error
                )),
            })
        } else {
            Ok(())
        }?;
    }

    Ok(())
}

fn get_blueprint_schema<'p>(
    package_definition: &'p PackageDefinition,
    package_address: PackageAddress,
    blueprint: &str,
) -> Result<&'p BlueprintSchema, InstructionSchemaValidationError> {
    package_definition.schema.blueprints.get(blueprint).ok_or(
        InstructionSchemaValidationError::InvalidBlueprint(package_address, blueprint.to_owned()),
    )
}

/// * An `Err` is returned if something is invalid about the arguments given to this method. As an
/// example: they've specified a method on the account blueprint that does not exist.
/// * A `None` is returned if the schema for this type is not well know. As an example: arbitrary
/// packages.
fn get_arguments_schema<'s>(
    invocation: Invocation,
) -> Result<
    Option<(LocalTypeIndex, &'s Schema<ScryptoCustomSchema>)>,
    InstructionSchemaValidationError,
> {
    let entity_type =
        if let Some(entity_type) = invocation.global_address().as_node_id().entity_type() {
            entity_type
        } else {
            return Err(InstructionSchemaValidationError::InvalidAddress(
                invocation.global_address(),
            ));
        };

    let blueprint_schema = match invocation {
        Invocation::Function(package_address @ PACKAGE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&PACKAGE_PACKAGE_DEFINITION, package_address, &blueprint)
                .map(Some)?
        }
        Invocation::Function(package_address @ RESOURCE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&RESOURCE_PACKAGE_DEFINITION, package_address, blueprint)
                .map(Some)?
        }
        Invocation::Function(package_address @ ACCOUNT_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&ACCOUNT_PACKAGE_DEFINITION, package_address, blueprint)
                .map(Some)?
        }
        Invocation::Function(package_address @ IDENTITY_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&IDENTITY_PACKAGE_DEFINITION, package_address, blueprint)
                .map(Some)?
        }
        Invocation::Function(package_address @ CONSENSUS_MANAGER_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                &CONSENSUS_MANAGER_PACKAGE_DEFINITION,
                package_address,
                blueprint,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ ACCESS_CONTROLLER_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                &ACCESS_CONTROLLER_PACKAGE_DEFINITION,
                package_address,
                blueprint,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ POOL_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&POOL_PACKAGE_DEFINITION, package_address, blueprint).map(Some)?
        }
        Invocation::Function(package_address @ TRANSACTION_PROCESSOR_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                &TRANSACTION_PROCESSOR_PACKAGE_DEFINITION,
                package_address,
                blueprint,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ METADATA_MODULE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&METADATA_PACKAGE_DEFINITION, package_address, blueprint)
                .map(Some)?
        }
        Invocation::Function(package_address @ ROYALTY_MODULE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&ROYALTIES_PACKAGE_DEFINITION, package_address, blueprint)
                .map(Some)?
        }
        Invocation::Function(package_address @ ACCESS_RULES_MODULE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(&ACCESS_RULES_PACKAGE_DEFINITION, package_address, blueprint)
                .map(Some)?
        }
        Invocation::Function(..) => None,
        Invocation::Method(_, ObjectModuleId::Main, _) => match entity_type {
            EntityType::GlobalPackage => PACKAGE_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(PACKAGE_BLUEPRINT),

            EntityType::GlobalConsensusManager => CONSENSUS_MANAGER_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(CONSENSUS_MANAGER_BLUEPRINT),
            EntityType::GlobalValidator => CONSENSUS_MANAGER_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(VALIDATOR_BLUEPRINT),

            EntityType::GlobalAccount
            | EntityType::InternalAccount
            | EntityType::GlobalVirtualEd25519Account
            | EntityType::GlobalVirtualSecp256k1Account => ACCOUNT_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(ACCOUNT_BLUEPRINT),

            EntityType::GlobalIdentity
            | EntityType::GlobalVirtualEd25519Identity
            | EntityType::GlobalVirtualSecp256k1Identity => IDENTITY_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(IDENTITY_BLUEPRINT),

            EntityType::GlobalAccessController => ACCESS_CONTROLLER_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(ACCESS_CONTROLLER_BLUEPRINT),

            EntityType::GlobalOneResourcePool => POOL_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(ONE_RESOURCE_POOL_BLUEPRINT_IDENT),
            EntityType::GlobalTwoResourcePool => POOL_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(TWO_RESOURCE_POOL_BLUEPRINT_IDENT),
            EntityType::GlobalMultiResourcePool => POOL_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(MULTI_RESOURCE_POOL_BLUEPRINT_IDENT),

            EntityType::GlobalFungibleResourceManager => RESOURCE_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            EntityType::GlobalNonFungibleResourceManager => RESOURCE_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            EntityType::InternalFungibleVault => RESOURCE_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(FUNGIBLE_VAULT_BLUEPRINT),
            EntityType::InternalNonFungibleVault => RESOURCE_PACKAGE_DEFINITION
                .schema
                .blueprints
                .get(NON_FUNGIBLE_VAULT_BLUEPRINT),

            EntityType::GlobalGenericComponent
            | EntityType::InternalGenericComponent
            | EntityType::InternalKeyValueStore => None,
        },
        Invocation::Method(_, ObjectModuleId::Metadata, _) => METADATA_PACKAGE_DEFINITION
            .schema
            .blueprints
            .get(METADATA_BLUEPRINT),
        Invocation::Method(_, ObjectModuleId::AccessRules, _) => ACCESS_RULES_PACKAGE_DEFINITION
            .schema
            .blueprints
            .get(ACCESS_RULES_BLUEPRINT),
        Invocation::Method(_, ObjectModuleId::Royalty, _) => ROYALTIES_PACKAGE_DEFINITION
            .schema
            .blueprints
            .get(COMPONENT_ROYALTY_BLUEPRINT),
    };

    if let Some(blueprint_schema) = blueprint_schema {
        if let Some(function_schema) = blueprint_schema.functions.get(invocation.method()) {
            if function_schema.receiver.is_none() && invocation.is_function()
                || function_schema.receiver.is_some() && invocation.is_method()
            {
                Ok(Some((function_schema.input, &blueprint_schema.schema)))
            } else {
                Err(InstructionSchemaValidationError::InvalidReceiver)
            }
        } else {
            Err(InstructionSchemaValidationError::MethodNotFound(
                invocation.method().to_owned(),
            ))
        }
    } else {
        Ok(None)
    }
}

enum Invocation {
    Method(GlobalAddress, ObjectModuleId, String),
    Function(PackageAddress, String, String),
}

impl Invocation {
    fn method(&self) -> &str {
        match self {
            Self::Method(_, _, method) => method,
            Self::Function(_, _, method) => method,
        }
    }

    fn global_address(&self) -> GlobalAddress {
        match self {
            Self::Method(global_address, ..) => *global_address,
            Self::Function(package_address, ..) => (*package_address).into(),
        }
    }

    fn is_function(&self) -> bool {
        match self {
            Self::Function(..) => true,
            Self::Method(..) => false,
        }
    }

    fn is_method(&self) -> bool {
        match self {
            Self::Method(..) => true,
            Self::Function(..) => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocatedInstructionSchemaValidationError {
    pub instruction_index: usize,
    pub cause: InstructionSchemaValidationError,
}

#[derive(Clone, Debug)]
pub enum InstructionSchemaValidationError {
    MethodNotFound(String),
    SchemaValidationError(String),

    InvalidAddress(GlobalAddress),
    InvalidBlueprint(PackageAddress, String),
    InvalidReceiver,
}
