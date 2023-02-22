use crate::blueprints::access_controller::AccessControllerSubstate;
use crate::blueprints::account::AccountSubstate;
use crate::blueprints::clock::*;
use crate::blueprints::epoch_manager::*;
use crate::blueprints::logger::LoggerSubstate;
use crate::blueprints::resource::*;
use crate::blueprints::transaction_runtime::TransactionRuntimeSubstate;
use crate::system::global::GlobalSubstate;
use crate::system::node_modules::access_rules::*;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_substates::*;
use crate::system::type_info::PackageCodeTypeSubstate;
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, BucketOffset, ComponentOffset, EpochManagerOffset, GlobalOffset,
    NonFungibleStoreOffset, PackageOffset, ProofOffset, ResourceManagerOffset, SubstateOffset,
    VaultOffset, WorktopOffset,
};

use super::events::EventStoreSubstate;

#[derive(Debug)]
pub enum RENodeModuleInit {
    TypeInfo(TypeInfoSubstate),
    Metadata(MetadataSubstate),
    ObjectAccessRulesChain(ObjectAccessRulesChainSubstate),
    ComponentRoyalty(
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
    ),
    PackageRoyalty(
        PackageRoyaltyConfigSubstate,
        PackageRoyaltyAccumulatorSubstate,
    ),
    PackageAccessRules(PackageAccessRulesSubstate),
}

impl RENodeModuleInit {
    pub fn to_substates(self) -> HashMap<SubstateOffset, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();
        match self {
            RENodeModuleInit::Metadata(metadata) => {
                substates.insert(
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                    metadata.into(),
                );
            }
            RENodeModuleInit::ObjectAccessRulesChain(access_rules) => {
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
                );
            }
            RENodeModuleInit::TypeInfo(type_info) => {
                substates.insert(
                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                    type_info.into(),
                );
            }
            RENodeModuleInit::ComponentRoyalty(config, accumulator) => {
                substates.insert(
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
                    config.into(),
                );
                substates.insert(
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                    accumulator.into(),
                );
            }
            RENodeModuleInit::PackageRoyalty(config, accumulator) => {
                substates.insert(
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
                    config.into(),
                );
                substates.insert(
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                    accumulator.into(),
                );
            }
            RENodeModuleInit::PackageAccessRules(access_rules) => {
                substates.insert(SubstateOffset::PackageAccessRules, access_rules.into());
            }
        }

        substates
    }
}

#[derive(Debug)]
pub enum RENodeInit {
    Global(GlobalSubstate),
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    AuthZoneStack(AuthZoneStackSubstate),
    Vault(VaultRuntimeSubstate),
    Worktop(WorktopSubstate),
    KeyValueStore,
    NonFungibleStore(NonFungibleStore),
    Identity(),
    Component(ComponentStateSubstate),
    Package(
        PackageInfoSubstate,
        PackageCodeTypeSubstate,
        PackageCodeSubstate,
    ),
    ResourceManager(ResourceManagerSubstate),
    EpochManager(
        EpochManagerSubstate,
        ValidatorSetSubstate,
        ValidatorSetSubstate,
    ),
    Validator(ValidatorSubstate),
    Clock(CurrentTimeRoundedToMinutesSubstate),
    TransactionRuntime(TransactionRuntimeSubstate),
    Logger(LoggerSubstate),
    AccessController(AccessControllerSubstate),
    Account(AccountSubstate),
    EventStore(EventStoreSubstate),
}

impl RENodeInit {
    pub fn to_substates(self) -> HashMap<SubstateOffset, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();
        match self {
            RENodeInit::Bucket(bucket) => {
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                    RuntimeSubstate::Bucket(bucket),
                );
            }
            RENodeInit::Proof(proof) => {
                substates.insert(
                    SubstateOffset::Proof(ProofOffset::Proof),
                    RuntimeSubstate::Proof(proof),
                );
            }
            RENodeInit::AuthZoneStack(auth_zone) => {
                substates.insert(
                    SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                    RuntimeSubstate::AuthZoneStack(auth_zone),
                );
            }
            RENodeInit::Global(global_node) => {
                substates.insert(
                    SubstateOffset::Global(GlobalOffset::Global),
                    RuntimeSubstate::Global(global_node),
                );
            }
            RENodeInit::Vault(vault) => {
                substates.insert(SubstateOffset::Vault(VaultOffset::Vault), vault.into());
            }
            RENodeInit::KeyValueStore => {}
            RENodeInit::Identity() => {}
            RENodeInit::Component(state) => {
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::State0),
                    state.into(),
                );
            }
            RENodeInit::Worktop(worktop) => {
                substates.insert(
                    SubstateOffset::Worktop(WorktopOffset::Worktop),
                    RuntimeSubstate::Worktop(worktop),
                );
            }
            RENodeInit::Logger(logger) => {
                substates.insert(
                    SubstateOffset::Logger(LoggerOffset::Logger),
                    RuntimeSubstate::Logger(logger),
                );
            }
            RENodeInit::Package(package_info, code_type, code) => {
                substates.insert(
                    SubstateOffset::Package(PackageOffset::Info),
                    package_info.into(),
                );
                substates.insert(
                    SubstateOffset::Package(PackageOffset::CodeType),
                    code_type.into(),
                );
                substates.insert(SubstateOffset::Package(PackageOffset::Code), code.into());
            }
            RENodeInit::ResourceManager(resource_manager) => {
                substates.insert(
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
                    resource_manager.into(),
                );
            }
            RENodeInit::Validator(validator) => {
                substates.insert(
                    SubstateOffset::Validator(ValidatorOffset::Validator),
                    validator.into(),
                );
            }
            RENodeInit::NonFungibleStore(non_fungible_store) => {
                for (id, non_fungible) in non_fungible_store.loaded_non_fungibles {
                    substates.insert(
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
                        non_fungible.into(),
                    );
                }
            }
            RENodeInit::EpochManager(
                epoch_manager,
                current_validator_set_substate,
                preparing_validator_set_substate,
            ) => {
                substates.insert(
                    SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
                    epoch_manager.into(),
                );
                substates.insert(
                    SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet),
                    current_validator_set_substate.into(),
                );
                substates.insert(
                    SubstateOffset::EpochManager(EpochManagerOffset::PreparingValidatorSet),
                    preparing_validator_set_substate.into(),
                );
            }
            RENodeInit::Clock(current_time_rounded_to_minutes_substate) => {
                substates.insert(
                    SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes),
                    current_time_rounded_to_minutes_substate.into(),
                );
            }
            RENodeInit::TransactionRuntime(transaction_hash) => {
                substates.insert(
                    SubstateOffset::TransactionRuntime(
                        TransactionRuntimeOffset::TransactionRuntime,
                    ),
                    transaction_hash.into(),
                );
            }
            RENodeInit::Account(account_substate) => {
                substates.insert(
                    SubstateOffset::Account(AccountOffset::Account),
                    account_substate.into(),
                );
            }
            RENodeInit::AccessController(access_controller) => {
                substates.insert(
                    SubstateOffset::AccessController(AccessControllerOffset::AccessController),
                    access_controller.into(),
                );
            }
            RENodeInit::EventStore(event_store_substate) => {
                substates.insert(
                    SubstateOffset::EventStore(EventStoreOffset::EventStore),
                    event_store_substate.into(),
                );
            }
        };

        substates
    }
}
