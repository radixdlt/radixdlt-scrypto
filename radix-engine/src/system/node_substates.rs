use super::global::GlobalSubstate;
use super::node_modules::access_rules::AuthZoneStackSubstate;
use super::node_modules::access_rules::ObjectAccessRulesChainSubstate;
use super::node_modules::metadata::MetadataSubstate;
use super::type_info::PackageCodeTypeSubstate;
use crate::blueprints::access_controller::AccessControllerSubstate;
use crate::blueprints::account::AccountSubstate;
use crate::blueprints::clock::CurrentTimeRoundedToMinutesSubstate;
use crate::blueprints::epoch_manager::EpochManagerSubstate;
use crate::blueprints::epoch_manager::ValidatorSetSubstate;
use crate::blueprints::epoch_manager::ValidatorSubstate;
use crate::blueprints::logger::LoggerSubstate;
use crate::blueprints::resource::BucketInfoSubstate;
use crate::blueprints::resource::FungibleProof;
use crate::blueprints::resource::NonFungibleProof;
use crate::blueprints::resource::NonFungibleSubstate;
use crate::blueprints::resource::ProofInfoSubstate;
use crate::blueprints::resource::ResourceManagerSubstate;
use crate::blueprints::resource::VaultInfoSubstate;
use crate::blueprints::resource::WorktopSubstate;
use crate::blueprints::transaction_runtime::TransactionRuntimeSubstate;
use crate::errors::*;
use crate::system::node_modules::access_rules::PackageAccessRulesSubstate;
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::{
    Address, ComponentOffset, KeyValueStoreOffset, NonFungibleStoreOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::blueprints::resource::LiquidNonFungibleResource;
use radix_engine_interface::blueprints::resource::LockedFungibleResource;
use radix_engine_interface::blueprints::resource::LockedNonFungibleResource;
use radix_engine_interface::data::IndexedScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PersistedSubstate {
    /* SELF */
    Global(GlobalSubstate),
    EpochManager(EpochManagerSubstate),
    ValidatorSet(ValidatorSetSubstate),
    Validator(ValidatorSubstate),
    CurrentTimeRoundedToMinutes(CurrentTimeRoundedToMinutesSubstate),
    ResourceManager(ResourceManagerSubstate),
    ComponentState(ComponentStateSubstate),
    PackageInfo(PackageInfoSubstate),
    PackageCodeType(PackageCodeTypeSubstate),
    WasmCode(WasmCodeSubstate),
    NativeCode(NativeCodeSubstate),
    Account(AccountSubstate),
    AccessController(AccessControllerSubstate),
    VaultInfo(VaultInfoSubstate),
    VaultLiquidFungible(LiquidFungibleResource),
    VaultLiquidNonFungible(LiquidNonFungibleResource),

    /* Type info */
    TypeInfo(TypeInfoSubstate),

    /* Access rules */
    AccessRulesChain(ObjectAccessRulesChainSubstate),
    PackageAccessRules(PackageAccessRulesSubstate),

    /* Metadata */
    Metadata(MetadataSubstate),

    /* Royalty */
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(ComponentRoyaltyAccumulatorSubstate),
    PackageRoyaltyConfig(PackageRoyaltyConfigSubstate),
    PackageRoyaltyAccumulator(PackageRoyaltyAccumulatorSubstate),

    /* KVStore entry */
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
}

impl PersistedSubstate {
    pub fn vault_info(&self) -> &VaultInfoSubstate {
        if let PersistedSubstate::VaultInfo(vault) = self {
            vault
        } else {
            panic!("Not a vault info");
        }
    }

    pub fn vault_liquid_fungible_mut(&mut self) -> &mut LiquidFungibleResource {
        if let PersistedSubstate::VaultLiquidFungible(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn vault_info_mut(&mut self) -> &mut VaultInfoSubstate {
        if let PersistedSubstate::VaultInfo(vault) = self {
            vault
        } else {
            panic!("Not a vault info");
        }
    }

    pub fn component_royalty_accumulator(&self) -> &ComponentRoyaltyAccumulatorSubstate {
        if let PersistedSubstate::ComponentRoyaltyAccumulator(state) = self {
            state
        } else {
            panic!("Not a component royalty accumulator");
        }
    }

    pub fn package_royalty_accumulator(&self) -> &PackageRoyaltyAccumulatorSubstate {
        if let PersistedSubstate::PackageRoyaltyAccumulator(state) = self {
            state
        } else {
            panic!("Not a package royalty accumulator");
        }
    }

    pub fn global(&self) -> &GlobalSubstate {
        if let PersistedSubstate::Global(state) = self {
            state
        } else {
            panic!("Not a global address substate");
        }
    }

    pub fn resource_manager(&self) -> &ResourceManagerSubstate {
        if let PersistedSubstate::ResourceManager(state) = self {
            state
        } else {
            panic!("Not a resource manager substate");
        }
    }
}

impl Into<VaultInfoSubstate> for PersistedSubstate {
    fn into(self) -> VaultInfoSubstate {
        if let PersistedSubstate::VaultInfo(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl Into<LiquidFungibleResource> for PersistedSubstate {
    fn into(self) -> LiquidFungibleResource {
        if let PersistedSubstate::VaultLiquidFungible(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl Into<LiquidNonFungibleResource> for PersistedSubstate {
    fn into(self) -> LiquidNonFungibleResource {
        if let PersistedSubstate::VaultLiquidNonFungible(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl PersistedSubstate {
    pub fn to_runtime(self) -> RuntimeSubstate {
        match self {
            PersistedSubstate::Global(value) => RuntimeSubstate::Global(value),
            PersistedSubstate::EpochManager(value) => RuntimeSubstate::EpochManager(value),
            PersistedSubstate::ValidatorSet(value) => RuntimeSubstate::ValidatorSet(value),
            PersistedSubstate::Validator(value) => RuntimeSubstate::Validator(value),
            PersistedSubstate::CurrentTimeRoundedToMinutes(value) => {
                RuntimeSubstate::CurrentTimeRoundedToMinutes(value)
            }
            PersistedSubstate::ResourceManager(value) => RuntimeSubstate::ResourceManager(value),
            PersistedSubstate::ComponentState(value) => RuntimeSubstate::ComponentState(value),
            PersistedSubstate::PackageInfo(value) => RuntimeSubstate::PackageInfo(value),
            PersistedSubstate::PackageCodeType(value) => RuntimeSubstate::PackageCodeType(value),
            PersistedSubstate::WasmCode(value) => RuntimeSubstate::WasmCode(value),
            PersistedSubstate::NativeCode(value) => RuntimeSubstate::NativeCode(value),
            PersistedSubstate::VaultInfo(value) => RuntimeSubstate::VaultInfo(value),
            PersistedSubstate::VaultLiquidFungible(value) => {
                RuntimeSubstate::VaultLiquidFungible(value)
            }
            PersistedSubstate::VaultLiquidNonFungible(value) => {
                RuntimeSubstate::VaultLiquidNonFungible(value)
            }
            PersistedSubstate::NonFungible(value) => RuntimeSubstate::NonFungible(value),
            PersistedSubstate::KeyValueStoreEntry(value) => {
                RuntimeSubstate::KeyValueStoreEntry(value)
            }
            PersistedSubstate::Account(value) => RuntimeSubstate::Account(value),
            PersistedSubstate::AccessController(value) => RuntimeSubstate::AccessController(value),

            /* Node module starts */
            PersistedSubstate::TypeInfo(value) => RuntimeSubstate::TypeInfo(value),
            PersistedSubstate::AccessRulesChain(value) => RuntimeSubstate::AccessRulesChain(value),
            PersistedSubstate::PackageAccessRules(value) => {
                RuntimeSubstate::PackageAccessRules(value)
            }
            PersistedSubstate::Metadata(value) => RuntimeSubstate::Metadata(value),
            PersistedSubstate::PackageRoyaltyConfig(value) => {
                RuntimeSubstate::PackageRoyaltyConfig(value)
            }
            PersistedSubstate::PackageRoyaltyAccumulator(value) => {
                RuntimeSubstate::PackageRoyaltyAccumulator(value)
            }
            PersistedSubstate::ComponentRoyaltyConfig(value) => {
                RuntimeSubstate::ComponentRoyaltyConfig(value)
            }
            PersistedSubstate::ComponentRoyaltyAccumulator(value) => {
                RuntimeSubstate::ComponentRoyaltyAccumulator(value)
            } /* Node module ends */
        }
    }
}

pub enum PersistError {
    VaultLocked,
}

#[derive(Debug)]
pub enum RuntimeSubstate {
    /* SELF */
    Global(GlobalSubstate),
    EpochManager(EpochManagerSubstate),
    ValidatorSet(ValidatorSetSubstate),
    Validator(ValidatorSubstate),
    CurrentTimeRoundedToMinutes(CurrentTimeRoundedToMinutesSubstate),
    ResourceManager(ResourceManagerSubstate),
    ComponentState(ComponentStateSubstate),
    PackageInfo(PackageInfoSubstate),
    PackageCodeType(PackageCodeTypeSubstate),
    WasmCode(WasmCodeSubstate),
    NativeCode(NativeCodeSubstate),
    AuthZoneStack(AuthZoneStackSubstate),
    Worktop(WorktopSubstate),
    Logger(LoggerSubstate),
    TransactionRuntime(TransactionRuntimeSubstate),
    Account(AccountSubstate),
    AccessController(AccessControllerSubstate),

    VaultInfo(VaultInfoSubstate),
    VaultLiquidFungible(LiquidFungibleResource),
    VaultLiquidNonFungible(LiquidNonFungibleResource),
    VaultLockedFungible(LockedFungibleResource),
    VaultLockedNonFungible(LockedNonFungibleResource),

    BucketInfo(BucketInfoSubstate),
    BucketLiquidFungible(LiquidFungibleResource),
    BucketLiquidNonFungible(LiquidNonFungibleResource),
    BucketLockedFungible(LockedFungibleResource),
    BucketLockedNonFungible(LockedNonFungibleResource),

    ProofInfo(ProofInfoSubstate),
    FungibleProof(FungibleProof),
    NonFungibleProof(NonFungibleProof),

    /* Type info */
    TypeInfo(TypeInfoSubstate),

    /* Access rules */
    AccessRulesChain(ObjectAccessRulesChainSubstate),
    PackageAccessRules(PackageAccessRulesSubstate),

    /* Metadata */
    Metadata(MetadataSubstate),

    /* Royalty */
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(ComponentRoyaltyAccumulatorSubstate),
    PackageRoyaltyConfig(PackageRoyaltyConfigSubstate),
    PackageRoyaltyAccumulator(PackageRoyaltyAccumulatorSubstate),

    /* KVStore entry */
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
}

impl RuntimeSubstate {
    pub fn clone_to_persisted(&self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::Global(value) => PersistedSubstate::Global(value.clone()),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value.clone()),
            RuntimeSubstate::ValidatorSet(value) => PersistedSubstate::ValidatorSet(value.clone()),
            RuntimeSubstate::Validator(value) => PersistedSubstate::Validator(value.clone()),
            RuntimeSubstate::CurrentTimeRoundedToMinutes(value) => {
                PersistedSubstate::CurrentTimeRoundedToMinutes(value.clone())
            }
            RuntimeSubstate::ResourceManager(value) => {
                PersistedSubstate::ResourceManager(value.clone())
            }
            RuntimeSubstate::ComponentState(value) => {
                PersistedSubstate::ComponentState(value.clone())
            }
            RuntimeSubstate::PackageInfo(value) => PersistedSubstate::PackageInfo(value.clone()),
            RuntimeSubstate::PackageCodeType(value) => {
                PersistedSubstate::PackageCodeType(value.clone())
            }
            RuntimeSubstate::WasmCode(value) => PersistedSubstate::WasmCode(value.clone()),
            RuntimeSubstate::NativeCode(value) => PersistedSubstate::NativeCode(value.clone()),
            RuntimeSubstate::NonFungible(value) => PersistedSubstate::NonFungible(value.clone()),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                PersistedSubstate::KeyValueStoreEntry(value.clone())
            }
            RuntimeSubstate::VaultInfo(value) => PersistedSubstate::VaultInfo(value.clone()),
            RuntimeSubstate::VaultLiquidFungible(value) => {
                PersistedSubstate::VaultLiquidFungible(value.clone())
            }
            RuntimeSubstate::VaultLiquidNonFungible(value) => {
                PersistedSubstate::VaultLiquidNonFungible(value.clone())
            }
            RuntimeSubstate::Account(value) => PersistedSubstate::Account(value.clone()),
            RuntimeSubstate::AccessController(value) => {
                PersistedSubstate::AccessController(value.clone())
            }

            /* Node module starts */
            RuntimeSubstate::TypeInfo(value) => PersistedSubstate::TypeInfo(value.clone()),
            RuntimeSubstate::AccessRulesChain(value) => {
                PersistedSubstate::AccessRulesChain(value.clone())
            }
            RuntimeSubstate::PackageAccessRules(value) => {
                PersistedSubstate::PackageAccessRules(value.clone())
            }
            RuntimeSubstate::Metadata(value) => PersistedSubstate::Metadata(value.clone()),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                PersistedSubstate::ComponentRoyaltyConfig(value.clone())
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                PersistedSubstate::ComponentRoyaltyAccumulator(value.clone())
            }
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                PersistedSubstate::PackageRoyaltyConfig(value.clone())
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                PersistedSubstate::PackageRoyaltyAccumulator(value.clone())
            }
            /* Node module ends */
            RuntimeSubstate::AuthZoneStack(..)
            | RuntimeSubstate::BucketInfo(..)
            | RuntimeSubstate::BucketLiquidFungible(..)
            | RuntimeSubstate::BucketLiquidNonFungible(..)
            | RuntimeSubstate::BucketLockedFungible(..)
            | RuntimeSubstate::BucketLockedNonFungible(..)
            | RuntimeSubstate::VaultLockedFungible(..)
            | RuntimeSubstate::VaultLockedNonFungible(..)
            | RuntimeSubstate::ProofInfo(..)
            | RuntimeSubstate::FungibleProof(..)
            | RuntimeSubstate::NonFungibleProof(..)
            | RuntimeSubstate::Worktop(..)
            | RuntimeSubstate::Logger(..)
            | RuntimeSubstate::TransactionRuntime(..) => {
                panic!("Should not get here");
            }
        }
    }

    pub fn to_persisted(self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::Global(value) => PersistedSubstate::Global(value),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value),
            RuntimeSubstate::ValidatorSet(value) => PersistedSubstate::ValidatorSet(value),
            RuntimeSubstate::Validator(value) => PersistedSubstate::Validator(value),
            RuntimeSubstate::CurrentTimeRoundedToMinutes(value) => {
                PersistedSubstate::CurrentTimeRoundedToMinutes(value)
            }
            RuntimeSubstate::ResourceManager(value) => PersistedSubstate::ResourceManager(value),
            RuntimeSubstate::ComponentState(value) => PersistedSubstate::ComponentState(value),
            RuntimeSubstate::PackageInfo(value) => PersistedSubstate::PackageInfo(value),
            RuntimeSubstate::PackageCodeType(value) => PersistedSubstate::PackageCodeType(value),
            RuntimeSubstate::WasmCode(value) => PersistedSubstate::WasmCode(value),
            RuntimeSubstate::NativeCode(value) => PersistedSubstate::NativeCode(value),
            RuntimeSubstate::NonFungible(value) => PersistedSubstate::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                PersistedSubstate::KeyValueStoreEntry(value)
            }
            RuntimeSubstate::VaultInfo(value) => PersistedSubstate::VaultInfo(value),
            RuntimeSubstate::VaultLiquidFungible(value) => {
                PersistedSubstate::VaultLiquidFungible(value)
            }
            RuntimeSubstate::VaultLiquidNonFungible(value) => {
                PersistedSubstate::VaultLiquidNonFungible(value)
            }
            RuntimeSubstate::Account(value) => PersistedSubstate::Account(value),
            RuntimeSubstate::AccessController(value) => PersistedSubstate::AccessController(value),

            /* Node module starts */
            RuntimeSubstate::TypeInfo(value) => PersistedSubstate::TypeInfo(value),
            RuntimeSubstate::AccessRulesChain(value) => PersistedSubstate::AccessRulesChain(value),
            RuntimeSubstate::PackageAccessRules(value) => {
                PersistedSubstate::PackageAccessRules(value)
            }
            RuntimeSubstate::Metadata(value) => PersistedSubstate::Metadata(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                PersistedSubstate::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                PersistedSubstate::ComponentRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                PersistedSubstate::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                PersistedSubstate::PackageRoyaltyAccumulator(value)
            }
            /* Node module ends */
            RuntimeSubstate::AuthZoneStack(..)
            | RuntimeSubstate::BucketInfo(..)
            | RuntimeSubstate::BucketLiquidFungible(..)
            | RuntimeSubstate::BucketLiquidNonFungible(..)
            | RuntimeSubstate::BucketLockedFungible(..)
            | RuntimeSubstate::BucketLockedNonFungible(..)
            | RuntimeSubstate::VaultLockedFungible(..)
            | RuntimeSubstate::VaultLockedNonFungible(..)
            | RuntimeSubstate::ProofInfo(..)
            | RuntimeSubstate::FungibleProof(..)
            | RuntimeSubstate::NonFungibleProof(..)
            | RuntimeSubstate::Worktop(..)
            | RuntimeSubstate::Logger(..)
            | RuntimeSubstate::TransactionRuntime(..) => {
                panic!("Should not get here");
            }
        }
    }

    pub fn decode_from_buffer(
        offset: &SubstateOffset,
        buffer: &[u8],
    ) -> Result<Self, RuntimeError> {
        let substate = match offset {
            SubstateOffset::Component(ComponentOffset::State0) => {
                let substate =
                    scrypto_decode(buffer).map_err(|e| KernelError::SborDecodeError(e))?;
                RuntimeSubstate::ComponentState(substate)
            }
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate =
                    scrypto_decode(buffer).map_err(|e| KernelError::SborDecodeError(e))?;
                RuntimeSubstate::KeyValueStoreEntry(substate)
            }
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)) => {
                let substate =
                    scrypto_decode(buffer).map_err(|e| KernelError::SborDecodeError(e))?;
                RuntimeSubstate::NonFungible(substate)
            }
            offset => {
                return Err(RuntimeError::KernelError(KernelError::InvalidOffset(
                    offset.clone(),
                )))
            }
        };

        Ok(substate)
    }

    pub fn to_ref_mut(&mut self) -> SubstateRefMut {
        match self {
            RuntimeSubstate::Global(value) => SubstateRefMut::Global(value),
            RuntimeSubstate::PackageCodeType(value) => SubstateRefMut::PackageCodeType(value),
            RuntimeSubstate::EpochManager(value) => SubstateRefMut::EpochManager(value),
            RuntimeSubstate::ValidatorSet(value) => SubstateRefMut::ValidatorSet(value),
            RuntimeSubstate::Validator(value) => SubstateRefMut::Validator(value),
            RuntimeSubstate::CurrentTimeRoundedToMinutes(value) => {
                SubstateRefMut::CurrentTimeRoundedToMinutes(value)
            }
            RuntimeSubstate::AccessRulesChain(value) => SubstateRefMut::AccessRulesChain(value),
            RuntimeSubstate::Metadata(value) => SubstateRefMut::Metadata(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRefMut::ResourceManager(value),
            RuntimeSubstate::TypeInfo(value) => SubstateRefMut::TypeInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRefMut::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                SubstateRefMut::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                SubstateRefMut::ComponentRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageInfo(value) => SubstateRefMut::PackageInfo(value),
            RuntimeSubstate::PackageAccessRules(value) => SubstateRefMut::PackageAccessRules(value),
            RuntimeSubstate::WasmCode(value) => SubstateRefMut::WasmCode(value),
            RuntimeSubstate::NativeCode(value) => SubstateRefMut::NativeCode(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                SubstateRefMut::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                SubstateRefMut::PackageRoyaltyAccumulator(value)
            }
            RuntimeSubstate::VaultInfo(value) => SubstateRefMut::VaultInfo(value),
            RuntimeSubstate::VaultLiquidFungible(value) => {
                SubstateRefMut::VaultLiquidFungible(value)
            }
            RuntimeSubstate::VaultLiquidNonFungible(value) => {
                SubstateRefMut::VaultLiquidNonFungible(value)
            }
            RuntimeSubstate::VaultLockedFungible(value) => {
                SubstateRefMut::VaultLockedFungible(value)
            }
            RuntimeSubstate::VaultLockedNonFungible(value) => {
                SubstateRefMut::VaultLockedNonFungible(value)
            }
            RuntimeSubstate::BucketInfo(value) => SubstateRefMut::BucketInfo(value),
            RuntimeSubstate::BucketLiquidFungible(value) => {
                SubstateRefMut::BucketLiquidFungible(value)
            }
            RuntimeSubstate::BucketLiquidNonFungible(value) => {
                SubstateRefMut::BucketLiquidNonFungible(value)
            }
            RuntimeSubstate::BucketLockedFungible(value) => {
                SubstateRefMut::BucketLockedFungible(value)
            }
            RuntimeSubstate::BucketLockedNonFungible(value) => {
                SubstateRefMut::BucketLockedNonFungible(value)
            }
            RuntimeSubstate::ProofInfo(value) => SubstateRefMut::ProofInfo(value),
            RuntimeSubstate::FungibleProof(value) => SubstateRefMut::FungibleProof(value),
            RuntimeSubstate::NonFungibleProof(value) => SubstateRefMut::NonFungibleProof(value),
            RuntimeSubstate::NonFungible(value) => SubstateRefMut::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRefMut::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZoneStack(value) => SubstateRefMut::AuthZoneStack(value),
            RuntimeSubstate::Worktop(value) => SubstateRefMut::Worktop(value),
            RuntimeSubstate::Logger(value) => SubstateRefMut::Logger(value),
            RuntimeSubstate::TransactionRuntime(value) => SubstateRefMut::TransactionRuntime(value),
            RuntimeSubstate::Account(value) => SubstateRefMut::Account(value),
            RuntimeSubstate::AccessController(value) => SubstateRefMut::AccessController(value),
        }
    }

    pub fn to_ref(&self) -> SubstateRef {
        match self {
            RuntimeSubstate::Global(value) => SubstateRef::Global(value),
            RuntimeSubstate::TypeInfo(value) => SubstateRef::TypeInfo(value),
            RuntimeSubstate::EpochManager(value) => SubstateRef::EpochManager(value),
            RuntimeSubstate::ValidatorSet(value) => SubstateRef::ValidatorSet(value),
            RuntimeSubstate::Validator(value) => SubstateRef::Validator(value),
            RuntimeSubstate::CurrentTimeRoundedToMinutes(value) => {
                SubstateRef::CurrentTimeRoundedToMinutes(value)
            }
            RuntimeSubstate::AccessRulesChain(value) => SubstateRef::AccessRulesChain(value),
            RuntimeSubstate::Metadata(value) => SubstateRef::Metadata(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRef::ResourceManager(value),
            RuntimeSubstate::ComponentState(value) => SubstateRef::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                SubstateRef::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                SubstateRef::ComponentRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageInfo(value) => SubstateRef::PackageInfo(value),
            RuntimeSubstate::PackageAccessRules(value) => SubstateRef::PackageAccessRules(value),
            RuntimeSubstate::PackageCodeType(value) => SubstateRef::PackageCodeType(value),
            RuntimeSubstate::WasmCode(value) => SubstateRef::WasmCode(value),
            RuntimeSubstate::NativeCode(value) => SubstateRef::NativeCode(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                SubstateRef::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                SubstateRef::PackageRoyaltyAccumulator(value)
            }
            RuntimeSubstate::VaultInfo(value) => SubstateRef::VaultInfo(value),
            RuntimeSubstate::VaultLiquidFungible(value) => SubstateRef::VaultLiquidFungible(value),
            RuntimeSubstate::VaultLiquidNonFungible(value) => {
                SubstateRef::VaultLiquidNonFungible(value)
            }
            RuntimeSubstate::VaultLockedFungible(value) => SubstateRef::VaultLockedFungible(value),
            RuntimeSubstate::VaultLockedNonFungible(value) => {
                SubstateRef::VaultLockedNonFungible(value)
            }
            RuntimeSubstate::BucketInfo(value) => SubstateRef::BucketInfo(value),
            RuntimeSubstate::BucketLiquidFungible(value) => {
                SubstateRef::BucketLiquidFungible(value)
            }
            RuntimeSubstate::BucketLiquidNonFungible(value) => {
                SubstateRef::BucketLiquidNonFungible(value)
            }
            RuntimeSubstate::BucketLockedFungible(value) => {
                SubstateRef::BucketLockedFungible(value)
            }
            RuntimeSubstate::BucketLockedNonFungible(value) => {
                SubstateRef::BucketLockedNonFungible(value)
            }
            RuntimeSubstate::ProofInfo(value) => SubstateRef::ProofInfo(value),
            RuntimeSubstate::FungibleProof(value) => SubstateRef::FungibleProof(value),
            RuntimeSubstate::NonFungibleProof(value) => SubstateRef::NonFungibleProof(value),
            RuntimeSubstate::NonFungible(value) => SubstateRef::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZoneStack(value) => SubstateRef::AuthZoneStack(value),
            RuntimeSubstate::Worktop(value) => SubstateRef::Worktop(value),
            RuntimeSubstate::Logger(value) => SubstateRef::Logger(value),
            RuntimeSubstate::TransactionRuntime(value) => SubstateRef::TransactionRuntime(value),
            RuntimeSubstate::Account(value) => SubstateRef::Account(value),
            RuntimeSubstate::AccessController(value) => SubstateRef::AccessController(value),
        }
    }

    pub fn global(&self) -> &GlobalSubstate {
        if let RuntimeSubstate::Global(global) = self {
            global
        } else {
            panic!("Not a global RENode");
        }
    }

    pub fn vault_info(&self) -> &VaultInfoSubstate {
        if let RuntimeSubstate::VaultInfo(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn vault_info_mut(&mut self) -> &mut VaultInfoSubstate {
        if let RuntimeSubstate::VaultInfo(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn vault_liquid_fungible_mut(&mut self) -> &mut LiquidFungibleResource {
        if let RuntimeSubstate::VaultLiquidFungible(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn package_info(&self) -> &PackageInfoSubstate {
        if let RuntimeSubstate::PackageInfo(package) = self {
            package
        } else {
            panic!("Not a package info");
        }
    }

    pub fn package_royalty_accumulator(&self) -> &PackageRoyaltyAccumulatorSubstate {
        if let RuntimeSubstate::PackageRoyaltyAccumulator(acc) = self {
            acc
        } else {
            panic!("Not a package accumulator");
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleSubstate {
        if let RuntimeSubstate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a NonFungible");
        }
    }

    pub fn kv_store_entry(&self) -> &KeyValueStoreEntrySubstate {
        if let RuntimeSubstate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
        } else {
            panic!("Not a KVEntry");
        }
    }

    pub fn logger(&self) -> &LoggerSubstate {
        if let RuntimeSubstate::Logger(logger) = self {
            logger
        } else {
            panic!("Not a logger");
        }
    }

    pub fn metadata(&self) -> &MetadataSubstate {
        if let RuntimeSubstate::Metadata(metadata) = self {
            metadata
        } else {
            panic!("Not metadata");
        }
    }

    pub fn epoch_manager(&self) -> &EpochManagerSubstate {
        if let RuntimeSubstate::EpochManager(epoch_manager) = self {
            epoch_manager
        } else {
            panic!("Not epoch manager");
        }
    }

    pub fn validator_set(&self) -> &ValidatorSetSubstate {
        if let RuntimeSubstate::ValidatorSet(validator_set) = self {
            validator_set
        } else {
            panic!("Not a validator set");
        }
    }

    pub fn account(&self) -> &AccountSubstate {
        if let RuntimeSubstate::Account(account) = self {
            account
        } else {
            panic!("Not an account");
        }
    }

    pub fn access_rules_chain(&self) -> &ObjectAccessRulesChainSubstate {
        if let RuntimeSubstate::AccessRulesChain(access_rules_chain) = self {
            access_rules_chain
        } else {
            panic!("Not an access rules chain");
        }
    }

    pub fn access_controller(&self) -> &AccessControllerSubstate {
        if let RuntimeSubstate::AccessController(access_controller) = self {
            access_controller
        } else {
            panic!("Not an access controller");
        }
    }
}

impl Into<RuntimeSubstate> for ObjectAccessRulesChainSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::AccessRulesChain(self)
    }
}

impl Into<RuntimeSubstate> for MetadataSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::Metadata(self)
    }
}

impl Into<RuntimeSubstate> for EpochManagerSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::EpochManager(self)
    }
}

impl Into<RuntimeSubstate> for ValidatorSetSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ValidatorSet(self)
    }
}

impl Into<RuntimeSubstate> for ValidatorSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::Validator(self)
    }
}

impl Into<RuntimeSubstate> for CurrentTimeRoundedToMinutesSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::CurrentTimeRoundedToMinutes(self)
    }
}

impl Into<RuntimeSubstate> for WasmCodeSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::WasmCode(self)
    }
}

impl Into<RuntimeSubstate> for PackageInfoSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::PackageInfo(self)
    }
}

impl Into<RuntimeSubstate> for PackageAccessRulesSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::PackageAccessRules(self)
    }
}

impl Into<RuntimeSubstate> for NativeCodeSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::NativeCode(self)
    }
}

impl Into<RuntimeSubstate> for PackageCodeTypeSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::PackageCodeType(self)
    }
}

impl Into<RuntimeSubstate> for TypeInfoSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::TypeInfo(self)
    }
}

impl Into<RuntimeSubstate> for ComponentStateSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ComponentState(self)
    }
}

impl Into<RuntimeSubstate> for ResourceManagerSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ResourceManager(self)
    }
}

impl Into<RuntimeSubstate> for ComponentRoyaltyConfigSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ComponentRoyaltyConfig(self)
    }
}

impl Into<RuntimeSubstate> for ComponentRoyaltyAccumulatorSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ComponentRoyaltyAccumulator(self)
    }
}

impl Into<RuntimeSubstate> for VaultInfoSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::VaultInfo(self)
    }
}

impl Into<RuntimeSubstate> for NonFungibleSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::NonFungible(self)
    }
}

impl Into<RuntimeSubstate> for KeyValueStoreEntrySubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::KeyValueStoreEntry(self)
    }
}

impl Into<RuntimeSubstate> for PackageRoyaltyConfigSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::PackageRoyaltyConfig(self)
    }
}

impl Into<RuntimeSubstate> for PackageRoyaltyAccumulatorSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::PackageRoyaltyAccumulator(self)
    }
}

impl Into<RuntimeSubstate> for TransactionRuntimeSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::TransactionRuntime(self)
    }
}

impl Into<RuntimeSubstate> for AccountSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::Account(self)
    }
}

impl Into<RuntimeSubstate> for AccessControllerSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::AccessController(self)
    }
}

impl Into<LoggerSubstate> for RuntimeSubstate {
    fn into(self) -> LoggerSubstate {
        if let RuntimeSubstate::Logger(logger) = self {
            logger
        } else {
            panic!("Not a logger");
        }
    }
}

impl Into<TypeInfoSubstate> for RuntimeSubstate {
    fn into(self) -> TypeInfoSubstate {
        if let RuntimeSubstate::TypeInfo(component) = self {
            component
        } else {
            panic!("Not a component info");
        }
    }
}

impl Into<ComponentStateSubstate> for RuntimeSubstate {
    fn into(self) -> ComponentStateSubstate {
        if let RuntimeSubstate::ComponentState(component_state) = self {
            component_state
        } else {
            panic!("Not a component");
        }
    }
}

impl Into<ComponentRoyaltyConfigSubstate> for RuntimeSubstate {
    fn into(self) -> ComponentRoyaltyConfigSubstate {
        if let RuntimeSubstate::ComponentRoyaltyConfig(config) = self {
            config
        } else {
            panic!("Not a component royalty config");
        }
    }
}

impl Into<ComponentRoyaltyAccumulatorSubstate> for RuntimeSubstate {
    fn into(self) -> ComponentRoyaltyAccumulatorSubstate {
        if let RuntimeSubstate::ComponentRoyaltyAccumulator(vault) = self {
            vault
        } else {
            panic!("Not a component royalty accumulator");
        }
    }
}

impl Into<PackageRoyaltyConfigSubstate> for RuntimeSubstate {
    fn into(self) -> PackageRoyaltyConfigSubstate {
        if let RuntimeSubstate::PackageRoyaltyConfig(config) = self {
            config
        } else {
            panic!("Not a package royalty config");
        }
    }
}

impl Into<PackageRoyaltyAccumulatorSubstate> for RuntimeSubstate {
    fn into(self) -> PackageRoyaltyAccumulatorSubstate {
        if let RuntimeSubstate::PackageRoyaltyAccumulator(vault) = self {
            vault
        } else {
            panic!("Not a package royalty accumulator");
        }
    }
}

impl Into<ResourceManagerSubstate> for RuntimeSubstate {
    fn into(self) -> ResourceManagerSubstate {
        if let RuntimeSubstate::ResourceManager(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<WasmCodeSubstate> for RuntimeSubstate {
    fn into(self) -> WasmCodeSubstate {
        if let RuntimeSubstate::WasmCode(package) = self {
            package
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<NonFungibleSubstate> for RuntimeSubstate {
    fn into(self) -> NonFungibleSubstate {
        if let RuntimeSubstate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a non-fungible wrapper");
        }
    }
}

impl Into<KeyValueStoreEntrySubstate> for RuntimeSubstate {
    fn into(self) -> KeyValueStoreEntrySubstate {
        if let RuntimeSubstate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
        } else {
            panic!("Not a key value store entry wrapper");
        }
    }
}

impl Into<VaultInfoSubstate> for RuntimeSubstate {
    fn into(self) -> VaultInfoSubstate {
        if let RuntimeSubstate::VaultInfo(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl Into<BucketInfoSubstate> for RuntimeSubstate {
    fn into(self) -> BucketInfoSubstate {
        if let RuntimeSubstate::BucketInfo(substate) = self {
            substate
        } else {
            panic!("Not a bucket");
        }
    }
}

impl Into<LiquidFungibleResource> for RuntimeSubstate {
    fn into(self) -> LiquidFungibleResource {
        if let RuntimeSubstate::VaultLiquidFungible(substate) = self {
            substate
        } else if let RuntimeSubstate::BucketLiquidFungible(substate) = self {
            substate
        } else {
            panic!("Not a vault");
        }
    }
}

impl Into<LiquidNonFungibleResource> for RuntimeSubstate {
    fn into(self) -> LiquidNonFungibleResource {
        if let RuntimeSubstate::VaultLiquidNonFungible(substate) = self {
            substate
        } else if let RuntimeSubstate::BucketLiquidNonFungible(substate) = self {
            substate
        } else {
            panic!("Not a vault");
        }
    }
}

impl Into<EpochManagerSubstate> for RuntimeSubstate {
    fn into(self) -> EpochManagerSubstate {
        if let RuntimeSubstate::EpochManager(system) = self {
            system
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<ValidatorSubstate> for RuntimeSubstate {
    fn into(self) -> ValidatorSubstate {
        if let RuntimeSubstate::Validator(validator) = self {
            validator
        } else {
            panic!("Not a validator");
        }
    }
}

impl Into<GlobalSubstate> for RuntimeSubstate {
    fn into(self) -> GlobalSubstate {
        if let RuntimeSubstate::Global(substate) = self {
            substate
        } else {
            panic!("Not a global address substate");
        }
    }
}

impl Into<ProofInfoSubstate> for RuntimeSubstate {
    fn into(self) -> ProofInfoSubstate {
        if let RuntimeSubstate::ProofInfo(substate) = self {
            substate
        } else {
            panic!("Not a proof");
        }
    }
}

impl Into<ObjectAccessRulesChainSubstate> for RuntimeSubstate {
    fn into(self) -> ObjectAccessRulesChainSubstate {
        if let RuntimeSubstate::AccessRulesChain(substate) = self {
            substate
        } else {
            panic!("Not access rules");
        }
    }
}

impl Into<MetadataSubstate> for RuntimeSubstate {
    fn into(self) -> MetadataSubstate {
        if let RuntimeSubstate::Metadata(substate) = self {
            substate
        } else {
            panic!("Not metadata");
        }
    }
}

impl Into<ValidatorSetSubstate> for RuntimeSubstate {
    fn into(self) -> ValidatorSetSubstate {
        if let RuntimeSubstate::ValidatorSet(substate) = self {
            substate
        } else {
            panic!("Not a validator set");
        }
    }
}

impl Into<AuthZoneStackSubstate> for RuntimeSubstate {
    fn into(self) -> AuthZoneStackSubstate {
        if let RuntimeSubstate::AuthZoneStack(substate) = self {
            substate
        } else {
            panic!("Not a auth zone stack");
        }
    }
}

impl Into<TransactionRuntimeSubstate> for RuntimeSubstate {
    fn into(self) -> TransactionRuntimeSubstate {
        if let RuntimeSubstate::TransactionRuntime(substate) = self {
            substate
        } else {
            panic!("Not a transaction runtime");
        }
    }
}

pub enum SubstateRef<'a> {
    AuthZoneStack(&'a AuthZoneStackSubstate),
    Worktop(&'a WorktopSubstate),
    Logger(&'a LoggerSubstate),
    ProofInfo(&'a ProofInfoSubstate),
    FungibleProof(&'a FungibleProof),
    NonFungibleProof(&'a NonFungibleProof),
    TypeInfo(&'a TypeInfoSubstate),
    ComponentState(&'a ComponentStateSubstate),
    ComponentRoyaltyConfig(&'a ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(&'a ComponentRoyaltyAccumulatorSubstate),
    NonFungible(&'a NonFungibleSubstate),
    KeyValueStoreEntry(&'a KeyValueStoreEntrySubstate),
    WasmCode(&'a WasmCodeSubstate),
    PackageInfo(&'a PackageInfoSubstate),
    NativeCode(&'a NativeCodeSubstate),
    PackageRoyaltyConfig(&'a PackageRoyaltyConfigSubstate),
    PackageRoyaltyAccumulator(&'a PackageRoyaltyAccumulatorSubstate),
    VaultInfo(&'a VaultInfoSubstate),
    VaultLiquidFungible(&'a LiquidFungibleResource),
    VaultLiquidNonFungible(&'a LiquidNonFungibleResource),
    VaultLockedFungible(&'a LockedFungibleResource),
    VaultLockedNonFungible(&'a LockedNonFungibleResource),
    BucketInfo(&'a BucketInfoSubstate),
    BucketLiquidFungible(&'a LiquidFungibleResource),
    BucketLiquidNonFungible(&'a LiquidNonFungibleResource),
    BucketLockedFungible(&'a LockedFungibleResource),
    BucketLockedNonFungible(&'a LockedNonFungibleResource),
    ResourceManager(&'a ResourceManagerSubstate),
    EpochManager(&'a EpochManagerSubstate),
    ValidatorSet(&'a ValidatorSetSubstate),
    Validator(&'a ValidatorSubstate),
    CurrentTimeRoundedToMinutes(&'a CurrentTimeRoundedToMinutesSubstate),
    AccessRulesChain(&'a ObjectAccessRulesChainSubstate),
    PackageAccessRules(&'a PackageAccessRulesSubstate),
    Metadata(&'a MetadataSubstate),
    Global(&'a GlobalSubstate),
    PackageCodeType(&'a PackageCodeTypeSubstate),
    TransactionRuntime(&'a TransactionRuntimeSubstate),
    Account(&'a AccountSubstate),
    AccessController(&'a AccessControllerSubstate),
}

impl<'a> SubstateRef<'a> {
    pub fn to_scrypto_value(&self) -> IndexedScryptoValue {
        match self {
            SubstateRef::Global(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::PackageCodeType(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::EpochManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::CurrentTimeRoundedToMinutes(value) => {
                IndexedScryptoValue::from_typed(*value)
            }
            SubstateRef::ResourceManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::TypeInfo(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentState(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentRoyaltyConfig(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentRoyaltyAccumulator(value) => {
                IndexedScryptoValue::from_typed(*value)
            }
            SubstateRef::PackageInfo(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::NativeCode(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::PackageRoyaltyConfig(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::PackageRoyaltyAccumulator(value) => {
                IndexedScryptoValue::from_typed(*value)
            }
            SubstateRef::NonFungible(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::KeyValueStoreEntry(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::AccessRulesChain(value) => IndexedScryptoValue::from_typed(*value),
            _ => panic!("Unsupported scrypto value"),
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleSubstate {
        match self {
            SubstateRef::NonFungible(non_fungible_substate) => *non_fungible_substate,
            _ => panic!("Not a non fungible"),
        }
    }

    pub fn epoch_manager(&self) -> &EpochManagerSubstate {
        match self {
            SubstateRef::EpochManager(epoch_manager_substate) => *epoch_manager_substate,
            _ => panic!("Not an epoch manager substate"),
        }
    }

    pub fn validator(&self) -> &ValidatorSubstate {
        match self {
            SubstateRef::Validator(substate) => *substate,
            _ => panic!("Not a validator substate"),
        }
    }

    pub fn component_state(&self) -> &ComponentStateSubstate {
        match self {
            SubstateRef::ComponentState(state) => *state,
            _ => panic!("Not a component state"),
        }
    }

    pub fn component_info(&self) -> &TypeInfoSubstate {
        match self {
            SubstateRef::TypeInfo(info) => *info,
            _ => panic!("Not a component info"),
        }
    }

    pub fn component_royalty_config(&self) -> &ComponentRoyaltyConfigSubstate {
        match self {
            SubstateRef::ComponentRoyaltyConfig(info) => *info,
            _ => panic!("Not a component royalty config"),
        }
    }

    pub fn component_royalty_accumulator(&self) -> &ComponentRoyaltyAccumulatorSubstate {
        match self {
            SubstateRef::ComponentRoyaltyAccumulator(info) => *info,
            _ => panic!("Not a component royalty accumulator"),
        }
    }

    pub fn package_royalty_config(&self) -> &PackageRoyaltyConfigSubstate {
        match self {
            SubstateRef::PackageRoyaltyConfig(info) => *info,
            _ => panic!("Not a package royalty config"),
        }
    }

    pub fn package_royalty_accumulator(&self) -> &PackageRoyaltyAccumulatorSubstate {
        match self {
            SubstateRef::PackageRoyaltyAccumulator(info) => *info,
            _ => panic!("Not a package royalty accumulator"),
        }
    }

    pub fn package_access_rules(&self) -> &PackageAccessRulesSubstate {
        match self {
            SubstateRef::PackageAccessRules(info) => *info,
            _ => panic!("Not package access rules"),
        }
    }

    pub fn proof_info(&self) -> &ProofInfoSubstate {
        match self {
            SubstateRef::ProofInfo(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn fungible_proof(&self) -> &FungibleProof {
        match self {
            SubstateRef::FungibleProof(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn non_fungible_proof(&self) -> &NonFungibleProof {
        match self {
            SubstateRef::NonFungibleProof(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn auth_zone_stack(&self) -> &AuthZoneStackSubstate {
        match self {
            SubstateRef::AuthZoneStack(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&self) -> &WorktopSubstate {
        match self {
            SubstateRef::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn bucket_info(&self) -> &BucketInfoSubstate {
        match self {
            SubstateRef::BucketInfo(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_liquid_fungible(&self) -> &LiquidFungibleResource {
        match self {
            SubstateRef::BucketLiquidFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_liquid_non_fungible(&self) -> &LiquidNonFungibleResource {
        match self {
            SubstateRef::BucketLiquidNonFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_locked_fungible(&self) -> &LockedFungibleResource {
        match self {
            SubstateRef::BucketLockedFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_locked_non_fungible(&self) -> &LockedNonFungibleResource {
        match self {
            SubstateRef::BucketLockedNonFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn vault_info(&self) -> &VaultInfoSubstate {
        match self {
            SubstateRef::VaultInfo(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_liquid_fungible(&self) -> &LiquidFungibleResource {
        match self {
            SubstateRef::VaultLiquidFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_liquid_non_fungible(&self) -> &LiquidNonFungibleResource {
        match self {
            SubstateRef::VaultLiquidNonFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_locked_fungible(&self) -> &LockedFungibleResource {
        match self {
            SubstateRef::VaultLockedFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_locked_non_fungible(&self) -> &LockedNonFungibleResource {
        match self {
            SubstateRef::VaultLockedNonFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn kv_store_entry(&self) -> &KeyValueStoreEntrySubstate {
        match self {
            SubstateRef::KeyValueStoreEntry(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManagerSubstate {
        match self {
            SubstateRef::ResourceManager(value) => *value,
            _ => panic!("Not a resource manager"),
        }
    }

    pub fn code_type(&self) -> &PackageCodeTypeSubstate {
        match self {
            SubstateRef::PackageCodeType(value) => *value,
            _ => panic!("Not code type"),
        }
    }

    pub fn code(&self) -> &WasmCodeSubstate {
        match self {
            SubstateRef::WasmCode(value) => *value,
            _ => panic!("Not wasm code"),
        }
    }

    pub fn package_info(&self) -> &PackageInfoSubstate {
        match self {
            SubstateRef::PackageInfo(value) => *value,
            _ => panic!("Not a package"),
        }
    }

    pub fn access_rules_chain(&self) -> &ObjectAccessRulesChainSubstate {
        match self {
            SubstateRef::AccessRulesChain(value) => *value,
            _ => panic!("Not access rules chain"),
        }
    }

    pub fn global_address(&self) -> &GlobalSubstate {
        match self {
            SubstateRef::Global(value) => *value,
            _ => panic!("Not a global address"),
        }
    }

    pub fn metadata(&self) -> &MetadataSubstate {
        match self {
            SubstateRef::Metadata(value) => *value,
            _ => panic!("Not metadata"),
        }
    }

    pub fn transaction_runtime(&self) -> &TransactionRuntimeSubstate {
        match self {
            SubstateRef::TransactionRuntime(value) => *value,
            _ => panic!("Not transaction runtime"),
        }
    }

    pub fn current_time_rounded_to_minutes(&self) -> &CurrentTimeRoundedToMinutesSubstate {
        match self {
            SubstateRef::CurrentTimeRoundedToMinutes(substate) => *substate,
            _ => panic!("Not a current time rounded to minutes substate ref"),
        }
    }

    pub fn account(&self) -> &AccountSubstate {
        match self {
            SubstateRef::Account(value) => *value,
            _ => panic!("Not an account"),
        }
    }

    pub fn access_controller(&self) -> &AccessControllerSubstate {
        match self {
            SubstateRef::AccessController(substate) => *substate,
            _ => panic!("Not an access controller substate"),
        }
    }

    pub fn references_and_owned_nodes(&self) -> (HashSet<RENodeId>, Vec<RENodeId>) {
        match self {
            SubstateRef::Global(global) => {
                let mut owned_nodes = Vec::new();
                match global {
                    GlobalSubstate::Resource(resource_manager_id) => {
                        owned_nodes.push(RENodeId::ResourceManager(*resource_manager_id))
                    }
                    GlobalSubstate::Component(component_id) => {
                        owned_nodes.push(RENodeId::Component(*component_id))
                    }
                    GlobalSubstate::Identity(identity_id) => {
                        owned_nodes.push(RENodeId::Identity(*identity_id))
                    }
                    GlobalSubstate::EpochManager(epoch_manager_id) => {
                        owned_nodes.push(RENodeId::EpochManager(*epoch_manager_id))
                    }
                    GlobalSubstate::Clock(clock_id) => owned_nodes.push(RENodeId::Clock(*clock_id)),
                    GlobalSubstate::Package(package_id) => {
                        owned_nodes.push(RENodeId::Package(*package_id))
                    }
                    GlobalSubstate::Validator(validator_id) => {
                        owned_nodes.push(RENodeId::Validator(*validator_id))
                    }
                    GlobalSubstate::Account(account_id) => {
                        owned_nodes.push(RENodeId::Account(*account_id))
                    }
                    GlobalSubstate::AccessController(access_controller_id) => {
                        owned_nodes.push(RENodeId::AccessController(*access_controller_id))
                    }
                };

                (HashSet::new(), owned_nodes)
            }
            SubstateRef::Worktop(worktop) => {
                let nodes = worktop
                    .resources
                    .values()
                    .map(|o| RENodeId::Bucket(o.bucket_id()))
                    .collect();
                (HashSet::new(), nodes)
            }
            SubstateRef::VaultInfo(vault) => {
                let mut references = HashSet::new();
                references.insert(RENodeId::Global(Address::Resource(vault.resource_address)));
                (references, Vec::new())
            }
            SubstateRef::ProofInfo(proof) => {
                let mut references = HashSet::new();
                references.insert(RENodeId::Global(Address::Resource(proof.resource_address)));
                (references, Vec::new())
            }
            SubstateRef::BucketInfo(bucket) => {
                let mut references = HashSet::new();
                references.insert(RENodeId::Global(Address::Resource(bucket.resource_address)));
                (references, Vec::new())
            }
            SubstateRef::PackageInfo(substate) => {
                let mut references = HashSet::new();
                for component_ref in &substate.dependent_components {
                    references.insert(RENodeId::Global(Address::Component(*component_ref)));
                }
                for resource_ref in &substate.dependent_resources {
                    references.insert(RENodeId::Global(Address::Resource(*resource_ref)));
                }
                (references, Vec::new())
            }
            SubstateRef::TypeInfo(substate) => {
                let mut references = HashSet::new();
                references.insert(RENodeId::Global(Address::Package(substate.package_address)));
                (references, Vec::new())
            }
            SubstateRef::ResourceManager(substate) => {
                let mut owned_nodes = Vec::new();
                if let Some(nf_store_id) = substate.nf_store_id {
                    owned_nodes.push(RENodeId::NonFungibleStore(nf_store_id));
                }
                (HashSet::new(), owned_nodes)
            }
            SubstateRef::Validator(substate) => {
                let mut references = HashSet::new();
                let mut owned_nodes = Vec::new();
                references.insert(RENodeId::Global(Address::Component(substate.manager)));
                references.insert(RENodeId::Global(Address::Component(substate.address)));
                references.insert(RENodeId::Global(Address::Resource(substate.unstake_nft)));
                references.insert(RENodeId::Global(Address::Resource(
                    substate.liquidity_token,
                )));
                owned_nodes.push(RENodeId::Vault(substate.stake_xrd_vault_id));
                owned_nodes.push(RENodeId::Vault(substate.pending_xrd_withdraw_vault_id));
                (references, owned_nodes)
            }
            SubstateRef::AccessRulesChain(substate) => {
                let (_, _, owns, refs) = IndexedScryptoValue::from_typed(&substate).unpack();
                (refs, owns)
            }
            SubstateRef::AccessController(substate) => {
                let mut owned_nodes = Vec::new();
                owned_nodes.push(RENodeId::Vault(substate.controlled_asset));
                (HashSet::new(), owned_nodes)
            }
            SubstateRef::PackageRoyaltyAccumulator(substate) => {
                let mut owned_nodes = Vec::new();
                owned_nodes.push(RENodeId::Vault(substate.royalty.vault_id()));
                (HashSet::new(), owned_nodes)
            }
            SubstateRef::ComponentState(substate) => {
                let (_, _, owns, refs) = IndexedScryptoValue::from_slice(&substate.raw)
                    .unwrap()
                    .unpack();
                (refs, owns)
            }
            SubstateRef::ComponentRoyaltyAccumulator(substate) => {
                let mut owned_nodes = Vec::new();
                owned_nodes.push(RENodeId::Vault(substate.royalty.vault_id()));
                (HashSet::new(), owned_nodes)
            }
            SubstateRef::KeyValueStoreEntry(substate) => {
                (substate.global_references(), substate.owned_node_ids())
            }
            SubstateRef::NonFungible(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|non_fungible| IndexedScryptoValue::from_typed(non_fungible));
                if let Some(scrypto_value) = maybe_scrypto_value {
                    let (_, _, owns, refs) = scrypto_value.unpack();
                    (refs, owns)
                } else {
                    (HashSet::new(), Vec::new())
                }
            }
            SubstateRef::Account(substate) => {
                let mut owned_nodes = Vec::new();
                owned_nodes.push(RENodeId::KeyValueStore(
                    substate.vaults.key_value_store_id(),
                ));
                (HashSet::new(), owned_nodes)
            }
            _ => (HashSet::new(), Vec::new()),
        }
    }
}

pub enum SubstateRefMut<'a> {
    TypeInfo(&'a mut TypeInfoSubstate),
    ComponentState(&'a mut ComponentStateSubstate),
    ComponentRoyaltyConfig(&'a mut ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(&'a mut ComponentRoyaltyAccumulatorSubstate),
    PackageInfo(&'a mut PackageInfoSubstate),
    PackageCodeType(&'a mut PackageCodeTypeSubstate),
    WasmCode(&'a mut WasmCodeSubstate),
    NativeCode(&'a mut NativeCodeSubstate),
    PackageRoyaltyConfig(&'a mut PackageRoyaltyConfigSubstate),
    PackageRoyaltyAccumulator(&'a mut PackageRoyaltyAccumulatorSubstate),
    PackageAccessRules(&'a mut PackageAccessRulesSubstate),
    NonFungible(&'a mut NonFungibleSubstate),
    KeyValueStoreEntry(&'a mut KeyValueStoreEntrySubstate),
    VaultInfo(&'a mut VaultInfoSubstate),
    VaultLiquidFungible(&'a mut LiquidFungibleResource),
    VaultLiquidNonFungible(&'a mut LiquidNonFungibleResource),
    VaultLockedFungible(&'a mut LockedFungibleResource),
    VaultLockedNonFungible(&'a mut LockedNonFungibleResource),
    BucketInfo(&'a mut BucketInfoSubstate),
    BucketLiquidFungible(&'a mut LiquidFungibleResource),
    BucketLiquidNonFungible(&'a mut LiquidNonFungibleResource),
    BucketLockedFungible(&'a mut LockedFungibleResource),
    BucketLockedNonFungible(&'a mut LockedNonFungibleResource),
    ResourceManager(&'a mut ResourceManagerSubstate),
    EpochManager(&'a mut EpochManagerSubstate),
    ValidatorSet(&'a mut ValidatorSetSubstate),
    Validator(&'a mut ValidatorSubstate),
    CurrentTimeRoundedToMinutes(&'a mut CurrentTimeRoundedToMinutesSubstate),
    AccessRulesChain(&'a mut ObjectAccessRulesChainSubstate),
    Metadata(&'a mut MetadataSubstate),
    Global(&'a mut GlobalSubstate),
    ProofInfo(&'a mut ProofInfoSubstate),
    FungibleProof(&'a mut FungibleProof),
    NonFungibleProof(&'a mut NonFungibleProof),
    Worktop(&'a mut WorktopSubstate),
    Logger(&'a mut LoggerSubstate),
    TransactionRuntime(&'a mut TransactionRuntimeSubstate),
    AuthZoneStack(&'a mut AuthZoneStackSubstate),
    AuthZone(&'a mut AuthZoneStackSubstate),
    Account(&'a mut AccountSubstate),
    AccessController(&'a mut AccessControllerSubstate),
}

impl<'a> SubstateRefMut<'a> {
    pub fn auth_zone_stack(&mut self) -> &mut AuthZoneStackSubstate {
        match self {
            SubstateRefMut::AuthZoneStack(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&mut self) -> &mut WorktopSubstate {
        match self {
            SubstateRefMut::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn vault_info(&mut self) -> &mut VaultInfoSubstate {
        match self {
            SubstateRefMut::VaultInfo(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_liquid_fungible(&mut self) -> &mut LiquidFungibleResource {
        match self {
            SubstateRefMut::VaultLiquidFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_liquid_non_fungible(&mut self) -> &mut LiquidNonFungibleResource {
        match self {
            SubstateRefMut::VaultLiquidNonFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_locked_fungible(&mut self) -> &mut LockedFungibleResource {
        match self {
            SubstateRefMut::VaultLockedFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn vault_locked_non_fungible(&mut self) -> &mut LockedNonFungibleResource {
        match self {
            SubstateRefMut::VaultLockedNonFungible(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn proof_info(&mut self) -> &mut ProofInfoSubstate {
        match self {
            SubstateRefMut::ProofInfo(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn bucket_info(&mut self) -> &mut BucketInfoSubstate {
        match self {
            SubstateRefMut::BucketInfo(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_liquid_fungible(&mut self) -> &mut LiquidFungibleResource {
        match self {
            SubstateRefMut::BucketLiquidFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_liquid_non_fungible(&mut self) -> &mut LiquidNonFungibleResource {
        match self {
            SubstateRefMut::BucketLiquidNonFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_locked_fungible(&mut self) -> &mut LockedFungibleResource {
        match self {
            SubstateRefMut::BucketLockedFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn bucket_locked_non_fungible(&mut self) -> &mut LockedNonFungibleResource {
        match self {
            SubstateRefMut::BucketLockedNonFungible(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn non_fungible(&mut self) -> &mut NonFungibleSubstate {
        match self {
            SubstateRefMut::NonFungible(value) => *value,
            _ => panic!("Not a non fungible"),
        }
    }

    pub fn resource_manager(&mut self) -> &mut ResourceManagerSubstate {
        match self {
            SubstateRefMut::ResourceManager(value) => *value,
            _ => panic!("Not resource manager"),
        }
    }

    pub fn kv_store_entry(&mut self) -> &mut KeyValueStoreEntrySubstate {
        match self {
            SubstateRefMut::KeyValueStoreEntry(value) => *value,
            _ => panic!("Not a key value store entry"),
        }
    }

    pub fn component_state(&mut self) -> &mut ComponentStateSubstate {
        match self {
            SubstateRefMut::ComponentState(value) => *value,
            _ => panic!("Not component state"),
        }
    }

    pub fn component_info(&mut self) -> &mut TypeInfoSubstate {
        match self {
            SubstateRefMut::TypeInfo(value) => *value,
            _ => panic!("Not component info"),
        }
    }

    pub fn component_royalty_config(&mut self) -> &mut ComponentRoyaltyConfigSubstate {
        match self {
            SubstateRefMut::ComponentRoyaltyConfig(value) => *value,
            _ => panic!("Not component royalty config"),
        }
    }

    pub fn component_royalty_accumulator(&mut self) -> &mut ComponentRoyaltyAccumulatorSubstate {
        match self {
            SubstateRefMut::ComponentRoyaltyAccumulator(value) => *value,
            _ => panic!("Not component royalty accumulator"),
        }
    }

    pub fn package_royalty_config(&mut self) -> &mut PackageRoyaltyConfigSubstate {
        match self {
            SubstateRefMut::PackageRoyaltyConfig(value) => *value,
            _ => panic!("Not package royalty config"),
        }
    }

    pub fn package_royalty_accumulator(&mut self) -> &mut PackageRoyaltyAccumulatorSubstate {
        match self {
            SubstateRefMut::PackageRoyaltyAccumulator(value) => *value,
            _ => panic!("Not package royalty accumulator"),
        }
    }

    pub fn epoch_manager(&mut self) -> &mut EpochManagerSubstate {
        match self {
            SubstateRefMut::EpochManager(value) => *value,
            _ => panic!("Not epoch manager"),
        }
    }

    pub fn validator(&mut self) -> &mut ValidatorSubstate {
        match self {
            SubstateRefMut::Validator(value) => *value,
            _ => panic!("Not validator"),
        }
    }

    pub fn validator_set(&mut self) -> &mut ValidatorSetSubstate {
        match self {
            SubstateRefMut::ValidatorSet(value) => *value,
            _ => panic!("Not a validator set"),
        }
    }

    pub fn current_time_rounded_to_minutes(&mut self) -> &mut CurrentTimeRoundedToMinutesSubstate {
        match self {
            SubstateRefMut::CurrentTimeRoundedToMinutes(value) => *value,
            _ => panic!("Not a current time rounded to minutes"),
        }
    }

    pub fn transaction_runtime(&mut self) -> &mut TransactionRuntimeSubstate {
        match self {
            SubstateRefMut::TransactionRuntime(value) => *value,
            _ => panic!("Not a transaction runtime"),
        }
    }

    pub fn logger(&mut self) -> &mut LoggerSubstate {
        match self {
            SubstateRefMut::Logger(value) => *value,
            _ => panic!("Not a logger"),
        }
    }

    pub fn access_rules_chain(&mut self) -> &mut ObjectAccessRulesChainSubstate {
        match self {
            SubstateRefMut::AccessRulesChain(value) => *value,
            _ => panic!("Not access rules"),
        }
    }

    pub fn metadata(&mut self) -> &mut MetadataSubstate {
        match self {
            SubstateRefMut::Metadata(value) => *value,
            _ => panic!("Not metadata"),
        }
    }

    pub fn access_controller(&mut self) -> &mut AccessControllerSubstate {
        match self {
            SubstateRefMut::AccessController(value) => *value,
            _ => panic!("Not access controller"),
        }
    }
}
