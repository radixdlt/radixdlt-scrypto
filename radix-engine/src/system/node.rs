use crate::blueprints::access_controller::AccessControllerSubstate;
use crate::blueprints::account::AccountSubstate;
use crate::blueprints::clock::*;
use crate::blueprints::epoch_manager::*;
use crate::blueprints::logger::LoggerSubstate;
use crate::blueprints::resource::*;
use crate::blueprints::transaction_runtime::TransactionRuntimeSubstate;
use crate::system::node_modules::access_rules::*;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_substates::*;
use crate::system::type_info::PackageCodeTypeSubstate;
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, BucketOffset, ComponentOffset, EpochManagerOffset, NonFungibleStoreOffset,
    PackageOffset, ProofOffset, ResourceManagerOffset, SubstateOffset, VaultOffset, WorktopOffset,
};
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::blueprints::resource::LiquidNonFungibleResource;
use radix_engine_interface::blueprints::resource::LockedFungibleResource;
use radix_engine_interface::blueprints::resource::LockedNonFungibleResource;

#[derive(Debug)]
pub enum RENodeModuleInit {
    /* Type info */
    TypeInfo(TypeInfoSubstate),

    /* Metadata */
    Metadata(MetadataSubstate),

    /* Access rules */
    ObjectAccessRulesChain(ObjectAccessRulesChainSubstate),
    PackageAccessRules(PackageAccessRulesSubstate),

    /* Royalty */
    ComponentRoyalty(
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
    ),
    PackageRoyalty(
        PackageRoyaltyConfigSubstate,
        PackageRoyaltyAccumulatorSubstate,
    ),
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
            RENodeModuleInit::PackageAccessRules(access_rules) => {
                substates.insert(SubstateOffset::PackageAccessRules, access_rules.into());
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
        }

        substates
    }
}

#[derive(Debug)]
pub enum RENodeInit {
    GlobalComponent(BTreeMap<SubstateOffset, RuntimeSubstate>),
    GlobalPackage(
        PackageInfoSubstate,
        PackageCodeTypeSubstate,
        PackageCodeSubstate,
    ),
    GlobalResourceManager(ResourceManagerSubstate),
    FungibleVault(VaultInfoSubstate, LiquidFungibleResource),
    NonFungibleVault(VaultInfoSubstate, LiquidNonFungibleResource),
    FungibleBucket(BucketInfoSubstate, LiquidFungibleResource),
    NonFungibleBucket(BucketInfoSubstate, LiquidNonFungibleResource),
    FungibleProof(ProofInfoSubstate, FungibleProof),
    NonFungibleProof(ProofInfoSubstate, NonFungibleProof),
    AuthZoneStack(AuthZoneStackSubstate),
    Worktop(WorktopSubstate),
    KeyValueStore,
    NonFungibleStore(NonFungibleStore),
    Identity(),
    Component(ComponentStateSubstate),
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
}

impl RENodeInit {
    pub fn to_substates(self) -> HashMap<SubstateOffset, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();
        match self {
            RENodeInit::FungibleVault(info, liquid) => {
                substates.insert(
                    SubstateOffset::Vault(VaultOffset::Info),
                    RuntimeSubstate::VaultInfo(info),
                );
                substates.insert(
                    SubstateOffset::Vault(VaultOffset::LiquidFungible),
                    RuntimeSubstate::VaultLiquidFungible(liquid),
                );
                substates.insert(
                    SubstateOffset::Vault(VaultOffset::LockedFungible),
                    RuntimeSubstate::VaultLockedFungible(LockedFungibleResource::new_empty()),
                );
            }
            RENodeInit::NonFungibleVault(info, liquid) => {
                substates.insert(
                    SubstateOffset::Vault(VaultOffset::Info),
                    RuntimeSubstate::VaultInfo(info),
                );
                substates.insert(
                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                    RuntimeSubstate::VaultLiquidNonFungible(liquid),
                );
                substates.insert(
                    SubstateOffset::Vault(VaultOffset::LockedNonFungible),
                    RuntimeSubstate::VaultLockedNonFungible(LockedNonFungibleResource::new_empty()),
                );
            }
            RENodeInit::FungibleBucket(info, liquid) => {
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::Info),
                    RuntimeSubstate::BucketInfo(info),
                );
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::LiquidFungible),
                    RuntimeSubstate::BucketLiquidFungible(liquid),
                );
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::LockedFungible),
                    RuntimeSubstate::BucketLockedFungible(LockedFungibleResource::new_empty()),
                );
            }
            RENodeInit::NonFungibleBucket(info, liquid) => {
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::Info),
                    RuntimeSubstate::BucketInfo(info),
                );
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
                    RuntimeSubstate::BucketLiquidNonFungible(liquid),
                );
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::LockedNonFungible),
                    RuntimeSubstate::BucketLockedNonFungible(LockedNonFungibleResource::new_empty()),
                );
            }
            RENodeInit::FungibleProof(info, proof) => {
                substates.insert(
                    SubstateOffset::Proof(ProofOffset::Info),
                    RuntimeSubstate::ProofInfo(info),
                );
                substates.insert(
                    SubstateOffset::Proof(ProofOffset::Fungible),
                    RuntimeSubstate::FungibleProof(proof),
                );
            }
            RENodeInit::NonFungibleProof(info, proof) => {
                substates.insert(
                    SubstateOffset::Proof(ProofOffset::Info),
                    RuntimeSubstate::ProofInfo(info),
                );
                substates.insert(
                    SubstateOffset::Proof(ProofOffset::NonFungible),
                    RuntimeSubstate::NonFungibleProof(proof),
                );
            }
            RENodeInit::AuthZoneStack(auth_zone) => {
                substates.insert(
                    SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                    RuntimeSubstate::AuthZoneStack(auth_zone),
                );
            }
            RENodeInit::GlobalComponent(component_substates) => {
                substates.extend(component_substates);
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
            RENodeInit::GlobalPackage(package_info, code_type, code) => {
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
            RENodeInit::GlobalResourceManager(resource_manager) => {
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
        };

        substates
    }
}
