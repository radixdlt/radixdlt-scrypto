use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, BucketOffset, ComponentOffset, EpochManagerOffset, GlobalOffset,
    KeyValueStoreOffset, NonFungibleStoreOffset, PackageOffset, ProofOffset, ResourceManagerOffset,
    SubstateOffset, VaultOffset, WorktopOffset,
};

#[derive(Debug)]
pub enum RENodeInit {
    Global(GlobalAddressSubstate),
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    AuthZoneStack(AuthZoneStackSubstate),
    FeeReserve(FeeReserveSubstate),
    Vault(VaultRuntimeSubstate),
    Worktop(WorktopSubstate),
    KeyValueStore(KeyValueStore),
    NonFungibleStore(NonFungibleStore),
    Identity(MetadataSubstate, AccessRulesChainSubstate),
    Component(
        ComponentInfoSubstate,
        ComponentStateSubstate,
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
        MetadataSubstate,
        AccessRulesChainSubstate,
    ),
    Package(
        PackageInfoSubstate,
        PackageRoyaltyConfigSubstate,
        PackageRoyaltyAccumulatorSubstate,
        MetadataSubstate,
        AccessRulesChainSubstate,
    ),
    ResourceManager(
        ResourceManagerSubstate,
        MetadataSubstate,
        AccessRulesChainSubstate,
        AccessRulesChainSubstate,
    ),
    EpochManager(
        EpochManagerSubstate,
        ValidatorSetSubstate,
        ValidatorSetSubstate,
        AccessRulesChainSubstate,
    ),
    Validator(
        ValidatorSubstate,
        MetadataSubstate,
        AccessRulesChainSubstate,
    ),
    Clock(
        CurrentTimeRoundedToMinutesSubstate,
        AccessRulesChainSubstate,
    ),
    TransactionRuntime(TransactionRuntimeSubstate),
    Logger(LoggerSubstate),
    AccessController(AccessControllerSubstate, AccessRulesChainSubstate),
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
            RENodeInit::KeyValueStore(store) => {
                for (k, v) in store.loaded_entries {
                    substates.insert(
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(k)),
                        v.into(),
                    );
                }
            }
            RENodeInit::Identity(metadata, access_rules) => {
                substates.insert(
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                    metadata.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
                );
            }
            RENodeInit::Component(
                info,
                state,
                royalty_config,
                royalty_accumulator,
                metadata,
                access_rules,
            ) => {
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::Info),
                    info.into(),
                );
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::State),
                    state.into(),
                );
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::RoyaltyConfig),
                    royalty_config.into(),
                );
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::RoyaltyAccumulator),
                    royalty_accumulator.into(),
                );
                substates.insert(
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                    metadata.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
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
            RENodeInit::Package(
                package_info,
                package_royalty_config,
                package_royalty_accumulator,
                metadata,
                access_rules,
            ) => {
                substates.insert(
                    SubstateOffset::Package(PackageOffset::Info),
                    package_info.into(),
                );
                substates.insert(
                    SubstateOffset::Package(PackageOffset::RoyaltyConfig),
                    package_royalty_config.into(),
                );
                substates.insert(
                    SubstateOffset::Package(PackageOffset::RoyaltyAccumulator),
                    package_royalty_accumulator.into(),
                );
                substates.insert(
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                    metadata.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
                );
            }
            RENodeInit::ResourceManager(
                resource_manager,
                metadata,
                access_rules,
                vault_access_rules,
            ) => {
                substates.insert(
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
                    resource_manager.into(),
                );
                substates.insert(
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                    metadata.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
                );
                // TODO: Figure out what the right abstraction is for vault access rules
                substates.insert(
                    SubstateOffset::VaultAccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    vault_access_rules.into(),
                );
            }
            RENodeInit::Validator(validator, metadata, access_rules) => {
                substates.insert(
                    SubstateOffset::Validator(ValidatorOffset::Validator),
                    validator.into(),
                );
                substates.insert(
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                    metadata.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
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
                access_rules,
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
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
                );
            }
            RENodeInit::Clock(current_time_rounded_to_minutes_substate, access_rules_substate) => {
                substates.insert(
                    SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes),
                    current_time_rounded_to_minutes_substate.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules_substate.into(),
                );
            }
            RENodeInit::FeeReserve(fee_reserve) => {
                substates.insert(
                    SubstateOffset::FeeReserve(FeeReserveOffset::FeeReserve),
                    fee_reserve.into(),
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
            RENodeInit::AccessController(access_controller, access_rules) => {
                substates.insert(
                    SubstateOffset::AccessController(AccessControllerOffset::AccessController),
                    access_controller.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                    access_rules.into(),
                );
            }
        }

        substates
    }
}
