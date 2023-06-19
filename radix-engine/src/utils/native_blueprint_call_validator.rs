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
use radix_engine_interface::schema::*;
use sbor::*;
use transaction::prelude::*;

pub fn validate_call_arguments_to_native_components(
    instructions: &[InstructionV1],
) -> Result<(), LocatedInstructionSchemaValidationError> {
    for (index, instruction) in instructions.iter().enumerate() {
        let (invocation, args) = match instruction {
            InstructionV1::CallFunction {
                package_address: DynamicPackageAddress::Static(address),
                blueprint_name,
                function_name,
                args,
            } => (
                Invocation::Function(
                    *address,
                    blueprint_name.to_owned(),
                    function_name.to_owned(),
                ),
                args,
            ),
            InstructionV1::CallMethod {
                address: DynamicGlobalAddress::Static(address),
                method_name,
                args,
            } => (
                Invocation::Method(*address, ObjectModuleId::Main, method_name.to_owned()),
                args,
            ),
            InstructionV1::CallMetadataMethod {
                address: DynamicGlobalAddress::Static(address),
                method_name,
                args,
            } => (
                Invocation::Method(*address, ObjectModuleId::Metadata, method_name.to_owned()),
                args,
            ),
            InstructionV1::CallRoyaltyMethod {
                address: DynamicGlobalAddress::Static(address),
                method_name,
                args,
            } => (
                Invocation::Method(*address, ObjectModuleId::Royalty, method_name.to_owned()),
                args,
            ),
            InstructionV1::CallAccessRulesMethod {
                address: DynamicGlobalAddress::Static(address),
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
            InstructionV1::CallDirectVaultMethod {
                address,
                method_name,
                args,
            } => (
                Invocation::DirectMethod(*address, method_name.to_owned()),
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

        if let Some((TypeRef::Static(local_type_index), schema)) = schema {
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
) -> Result<&'p BlueprintDefinitionInit, InstructionSchemaValidationError> {
    package_definition.blueprints.get(blueprint).ok_or(
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
    Option<(TypeRef<LocalTypeIndex>, &'s Schema<ScryptoCustomSchema>)>,
    InstructionSchemaValidationError,
> {
    let entity_type = invocation.entity_type();

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
        Invocation::Method(_, ObjectModuleId::Main, _) | Invocation::DirectMethod(..) => {
            match entity_type {
                EntityType::GlobalPackage => {
                    PACKAGE_PACKAGE_DEFINITION.blueprints.get(PACKAGE_BLUEPRINT)
                }

                EntityType::GlobalConsensusManager => CONSENSUS_MANAGER_PACKAGE_DEFINITION
                    .blueprints
                    .get(CONSENSUS_MANAGER_BLUEPRINT),
                EntityType::GlobalValidator => CONSENSUS_MANAGER_PACKAGE_DEFINITION
                    .blueprints
                    .get(VALIDATOR_BLUEPRINT),

                EntityType::GlobalAccount
                | EntityType::InternalAccount
                | EntityType::GlobalVirtualEd25519Account
                | EntityType::GlobalVirtualSecp256k1Account => {
                    ACCOUNT_PACKAGE_DEFINITION.blueprints.get(ACCOUNT_BLUEPRINT)
                }

                EntityType::GlobalIdentity
                | EntityType::GlobalVirtualEd25519Identity
                | EntityType::GlobalVirtualSecp256k1Identity => IDENTITY_PACKAGE_DEFINITION
                    .blueprints
                    .get(IDENTITY_BLUEPRINT),

                EntityType::GlobalAccessController => ACCESS_CONTROLLER_PACKAGE_DEFINITION
                    .blueprints
                    .get(ACCESS_CONTROLLER_BLUEPRINT),

                EntityType::GlobalOneResourcePool => POOL_PACKAGE_DEFINITION
                    .blueprints
                    .get(ONE_RESOURCE_POOL_BLUEPRINT_IDENT),
                EntityType::GlobalTwoResourcePool => POOL_PACKAGE_DEFINITION
                    .blueprints
                    .get(TWO_RESOURCE_POOL_BLUEPRINT_IDENT),
                EntityType::GlobalMultiResourcePool => POOL_PACKAGE_DEFINITION
                    .blueprints
                    .get(MULTI_RESOURCE_POOL_BLUEPRINT_IDENT),

                EntityType::GlobalTransactionTracker => TRANSACTION_TRACKER_PACKAGE_DEFINITION
                    .blueprints
                    .get(TRANSACTION_TRACKER_BLUEPRINT),

                EntityType::GlobalFungibleResourceManager => RESOURCE_PACKAGE_DEFINITION
                    .blueprints
                    .get(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
                EntityType::GlobalNonFungibleResourceManager => RESOURCE_PACKAGE_DEFINITION
                    .blueprints
                    .get(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
                EntityType::InternalFungibleVault => RESOURCE_PACKAGE_DEFINITION
                    .blueprints
                    .get(FUNGIBLE_VAULT_BLUEPRINT),
                EntityType::InternalNonFungibleVault => RESOURCE_PACKAGE_DEFINITION
                    .blueprints
                    .get(NON_FUNGIBLE_VAULT_BLUEPRINT),

                EntityType::GlobalGenericComponent
                | EntityType::InternalGenericComponent
                | EntityType::InternalKeyValueStore => None,
            }
        }
        Invocation::Method(_, ObjectModuleId::Metadata, _) => METADATA_PACKAGE_DEFINITION
            .blueprints
            .get(METADATA_BLUEPRINT),
        Invocation::Method(_, ObjectModuleId::AccessRules, _) => ACCESS_RULES_PACKAGE_DEFINITION
            .blueprints
            .get(ACCESS_RULES_BLUEPRINT),
        Invocation::Method(_, ObjectModuleId::Royalty, _) => ROYALTIES_PACKAGE_DEFINITION
            .blueprints
            .get(COMPONENT_ROYALTY_BLUEPRINT),
    };

    if let Some(blueprint_schema) = blueprint_schema {
        if let Some(function_schema) = blueprint_schema
            .schema
            .functions
            .functions
            .get(invocation.method())
        {
            if is_self_or_mut_self_receiver(&function_schema.receiver) && invocation.is_method()
                || is_direct_access_receiver(&function_schema.receiver)
                    && invocation.is_direct_access_method()
                || is_function_receiver(&function_schema.receiver) && invocation.is_function()
            {
                Ok(Some((
                    function_schema.input.clone(),
                    &blueprint_schema.schema.schema,
                )))
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

fn is_self_or_mut_self_receiver(receiver: &Option<ReceiverInfo>) -> bool {
    if let Some(ref receiver) = receiver {
        match (&receiver.receiver, receiver.ref_types) {
            (Receiver::SelfRef | Receiver::SelfRefMut, RefTypes::NORMAL) => true,
            _ => false,
        }
    } else {
        false
    }
}

fn is_direct_access_receiver(receiver: &Option<ReceiverInfo>) -> bool {
    if let Some(ref receiver) = receiver {
        match (&receiver.receiver, receiver.ref_types) {
            (Receiver::SelfRef | Receiver::SelfRefMut, RefTypes::DIRECT_ACCESS) => true,
            _ => false,
        }
    } else {
        false
    }
}

fn is_function_receiver(receiver: &Option<ReceiverInfo>) -> bool {
    receiver.is_none()
}

#[derive(Clone, Debug)]
enum Invocation {
    DirectMethod(InternalAddress, String),
    Method(GlobalAddress, ObjectModuleId, String),
    Function(PackageAddress, String, String),
}

impl Invocation {
    fn method(&self) -> &str {
        match self {
            Self::DirectMethod(_, method) => method,
            Self::Method(_, _, method) => method,
            Self::Function(_, _, method) => method,
        }
    }

    fn entity_type(&self) -> EntityType {
        match self {
            Self::DirectMethod(address, ..) => address.as_node_id().entity_type().unwrap(),
            Self::Method(address, ..) => address.as_node_id().entity_type().unwrap(),
            Self::Function(address, ..) => address.as_node_id().entity_type().unwrap(),
        }
    }

    fn is_function(&self) -> bool {
        match self {
            Self::Function(..) => true,
            Self::Method(..) | Self::DirectMethod(..) => false,
        }
    }

    fn is_method(&self) -> bool {
        match self {
            Self::Method(..) | Self::DirectMethod(..) => true,
            Self::Function(..) => false,
        }
    }

    fn is_direct_access_method(&self) -> bool {
        match self {
            Self::DirectMethod(..) => true,
            Self::Function(..) | Self::Method(..) => false,
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
