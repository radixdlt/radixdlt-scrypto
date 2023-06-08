use crate::blueprints::access_controller::*;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::consensus_manager::ConsensusManagerNativePackage;
use crate::blueprints::identity::*;
use crate::blueprints::package::*;
use crate::blueprints::pool::multi_resource_pool::*;
use crate::blueprints::pool::one_resource_pool::*;
use crate::blueprints::pool::two_resource_pool::*;
use crate::blueprints::pool::PoolNativePackage;
use crate::blueprints::resource::*;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::system::node_modules::access_rules::*;
use crate::system::node_modules::metadata::*;
use crate::system::node_modules::royalty::*;
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
) -> Result<(), ValidationError> {
    let mut package_definitions = LazyPackageDefinitions::new();

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

        let schema = get_arguments_schema(invocation, &mut package_definitions, index)?;
        if let Some((local_type_index, schema)) = schema {
            validate_payload_against_schema::<ManifestCustomExtension, _>(
                &manifest_encode(&args).unwrap(),
                schema,
                local_type_index,
                &(),
            )
            .map_err(|error| ValidationError::SchemaValidationError(index, format!("{:?}", error)))
        } else {
            Ok(())
        }?;
    }

    Ok(())
}

macro_rules! define_lazy_package_definitions {
    (
        struct $ident: ident {
            $(
                $field: ident = $value: expr
            ),* $(,)?
        }
    ) => {
        struct $ident {
            $(
                $field: Option<PackageDefinition>,
            )*
        }

        impl $ident {
            fn new() -> Self {
                Self {
                    $(
                        $field: None,
                    )*
                }
            }

            $(
                fn $field(&mut self) -> &PackageDefinition {
                    if let Some(ref schema) = self.$field {
                        schema
                    } else {
                        self.$field = Some($value);
                        self.$field()
                    }
                }
            )*
        }
    };
}

define_lazy_package_definitions! {
    struct LazyPackageDefinitions {
        consensus_manager_package = ConsensusManagerNativePackage::definition(),
        account_package = AccountNativePackage::definition(),
        identity_package = IdentityNativePackage::definition(),
        access_controller_package = AccessControllerNativePackage::definition(),
        pool_package = PoolNativePackage::definition(),
        resource_package = ResourceManagerNativePackage::definition(),
        package_package = PackageNativePackage::definition(),
        transaction_processor_package = TransactionProcessorNativePackage::definition(),
        metadata_package = MetadataNativePackage::definition(),
        royalties_package = RoyaltyNativePackage::definition(),
        access_rules_package = AccessRulesNativePackage::definition(),
    }
}

fn get_blueprint_schema<'p>(
    package_definition: &'p PackageDefinition,
    package_address: PackageAddress,
    blueprint: &str,
    instruction_index: usize,
) -> Result<&'p BlueprintSchema, ValidationError> {
    package_definition
        .schema
        .blueprints
        .get(blueprint)
        .ok_or(ValidationError::InvalidBlueprint(
            instruction_index,
            package_address,
            blueprint.to_owned(),
        ))
}

/// * An `Err` is returned if something is invalid about the arguments given to this method. As an
/// example: they've specified a method on the account blueprint that does not exist.
/// * A `None` is returned if the schema for this type is not well know. As an example: arbitrary
/// packages.
fn get_arguments_schema<'s>(
    invocation: Invocation,
    package_definitions: &'s mut LazyPackageDefinitions,
    instruction_index: usize,
) -> Result<Option<(LocalTypeIndex, &'s Schema<ScryptoCustomSchema>)>, ValidationError> {
    let entity_type =
        if let Some(entity_type) = invocation.global_address().as_node_id().entity_type() {
            entity_type
        } else {
            return Err(ValidationError::InvalidAddress(
                instruction_index,
                invocation.global_address(),
            ));
        };

    let blueprint_schema = match invocation {
        Invocation::Function(package_address @ PACKAGE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.package_package(),
                package_address,
                &blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ RESOURCE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.resource_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ ACCOUNT_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.account_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ IDENTITY_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.identity_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ CONSENSUS_MANAGER_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.consensus_manager_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ ACCESS_CONTROLLER_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.access_controller_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ POOL_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.pool_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ TRANSACTION_PROCESSOR_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.transaction_processor_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ METADATA_MODULE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.metadata_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ ROYALTY_MODULE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.royalties_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(package_address @ ACCESS_RULES_MODULE_PACKAGE, ref blueprint, _) => {
            get_blueprint_schema(
                package_definitions.access_rules_package(),
                package_address,
                blueprint,
                instruction_index,
            )
            .map(Some)?
        }
        Invocation::Function(..) => None,
        Invocation::Method(_, ObjectModuleId::Main, _) => match entity_type {
            EntityType::GlobalPackage => package_definitions
                .package_package()
                .schema
                .blueprints
                .get(PACKAGE_BLUEPRINT),

            EntityType::GlobalConsensusManager => package_definitions
                .consensus_manager_package()
                .schema
                .blueprints
                .get(CONSENSUS_MANAGER_BLUEPRINT),
            EntityType::GlobalValidator => package_definitions
                .consensus_manager_package()
                .schema
                .blueprints
                .get(VALIDATOR_BLUEPRINT),

            EntityType::GlobalAccount
            | EntityType::InternalAccount
            | EntityType::GlobalVirtualEd25519Account
            | EntityType::GlobalVirtualSecp256k1Account => package_definitions
                .account_package()
                .schema
                .blueprints
                .get(ACCOUNT_BLUEPRINT),

            EntityType::GlobalIdentity
            | EntityType::GlobalVirtualEd25519Identity
            | EntityType::GlobalVirtualSecp256k1Identity => package_definitions
                .identity_package()
                .schema
                .blueprints
                .get(IDENTITY_BLUEPRINT),

            EntityType::GlobalAccessController => package_definitions
                .access_controller_package()
                .schema
                .blueprints
                .get(ACCESS_CONTROLLER_BLUEPRINT),

            EntityType::GlobalOneResourcePool => package_definitions
                .pool_package()
                .schema
                .blueprints
                .get(ONE_RESOURCE_POOL_BLUEPRINT_IDENT),
            EntityType::GlobalTwoResourcePool => package_definitions
                .pool_package()
                .schema
                .blueprints
                .get(TWO_RESOURCE_POOL_BLUEPRINT_IDENT),
            EntityType::GlobalMultiResourcePool => package_definitions
                .pool_package()
                .schema
                .blueprints
                .get(MULTI_RESOURCE_POOL_BLUEPRINT_IDENT),

            EntityType::GlobalFungibleResourceManager => package_definitions
                .resource_package()
                .schema
                .blueprints
                .get(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            EntityType::GlobalNonFungibleResourceManager => package_definitions
                .resource_package()
                .schema
                .blueprints
                .get(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            EntityType::InternalFungibleVault => package_definitions
                .resource_package()
                .schema
                .blueprints
                .get(FUNGIBLE_VAULT_BLUEPRINT),
            EntityType::InternalNonFungibleVault => package_definitions
                .resource_package()
                .schema
                .blueprints
                .get(NON_FUNGIBLE_VAULT_BLUEPRINT),

            EntityType::GlobalGenericComponent
            | EntityType::InternalGenericComponent
            | EntityType::InternalKeyValueStore => None,
        },
        Invocation::Method(_, ObjectModuleId::Metadata, _) => package_definitions
            .metadata_package()
            .schema
            .blueprints
            .get(METADATA_BLUEPRINT),
        Invocation::Method(_, ObjectModuleId::AccessRules, _) => package_definitions
            .access_rules_package()
            .schema
            .blueprints
            .get(ACCESS_RULES_BLUEPRINT),
        Invocation::Method(_, ObjectModuleId::Royalty, _) => package_definitions
            .royalties_package()
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
                Err(ValidationError::InvalidReceiver(instruction_index))
            }
        } else {
            Err(ValidationError::MethodNotFound(
                instruction_index,
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

// `usize` here is the index of the instruction has has an error.
#[derive(Clone, Debug)]
pub enum ValidationError {
    MethodNotFound(usize, String),
    SchemaValidationError(usize, String),

    InvalidAddress(usize, GlobalAddress),
    InvalidBlueprint(usize, PackageAddress, String),
    InvalidReceiver(usize),
}
