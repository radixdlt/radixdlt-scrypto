use super::global::GlobalSubstate;
use super::node_modules::access_rules::AuthZoneStackSubstate;
use super::node_modules::access_rules::ObjectAccessRulesChainSubstate;
use super::node_modules::metadata::MetadataSubstate;
use crate::blueprints::access_controller::AccessControllerSubstate;
use crate::blueprints::account::AccountSubstate;
use crate::blueprints::clock::CurrentTimeRoundedToMinutesSubstate;
use crate::blueprints::epoch_manager::EpochManagerSubstate;
use crate::blueprints::epoch_manager::ValidatorSetSubstate;
use crate::blueprints::epoch_manager::ValidatorSubstate;
use crate::blueprints::logger::LoggerSubstate;
use crate::blueprints::resource::BucketSubstate;
use crate::blueprints::resource::NonFungibleSubstate;
use crate::blueprints::resource::ProofSubstate;
use crate::blueprints::resource::ResourceManagerSubstate;
use crate::blueprints::resource::VaultRuntimeSubstate;
use crate::blueprints::resource::VaultSubstate;
use crate::blueprints::resource::WorktopSubstate;
use crate::blueprints::transaction_runtime::TransactionRuntimeSubstate;
use crate::errors::*;
use crate::system::node_modules::access_rules::PackageAccessRulesSubstate;
use crate::system::type_info::TypeInfoSubstate;
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::{
    Address, ComponentOffset, KeyValueStoreOffset, NonFungibleStoreOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PersistedSubstate {
    Global(GlobalSubstate),
    TypeInfo(TypeInfoSubstate),
    EpochManager(EpochManagerSubstate),
    ValidatorSet(ValidatorSetSubstate),
    Validator(ValidatorSubstate),
    CurrentTimeRoundedToMinutes(CurrentTimeRoundedToMinutesSubstate),
    ResourceManager(ResourceManagerSubstate),
    AccessRulesChain(ObjectAccessRulesChainSubstate),
    Metadata(MetadataSubstate),
    ComponentInfo(ComponentInfoSubstate),
    ComponentState(ComponentStateSubstate),
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(ComponentRoyaltyAccumulatorSubstate),
    PackageInfo(PackageInfoSubstate),
    WasmCode(WasmCodeSubstate),
    NativePackageInfo(NativeCodeSubstate),
    PackageRoyaltyConfig(PackageRoyaltyConfigSubstate),
    PackageRoyaltyAccumulator(PackageRoyaltyAccumulatorSubstate),
    PackageAccessRules(PackageAccessRulesSubstate),
    Vault(VaultSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
    Account(AccountSubstate),
    AccessController(AccessControllerSubstate),
}

impl PersistedSubstate {
    pub fn vault(&self) -> &VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn vault_mut(&mut self) -> &mut VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
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

impl Into<VaultSubstate> for PersistedSubstate {
    fn into(self) -> VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
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
            PersistedSubstate::TypeInfo(value) => RuntimeSubstate::TypeInfo(value),
            PersistedSubstate::EpochManager(value) => RuntimeSubstate::EpochManager(value),
            PersistedSubstate::ValidatorSet(value) => RuntimeSubstate::ValidatorSet(value),
            PersistedSubstate::Validator(value) => RuntimeSubstate::Validator(value),
            PersistedSubstate::CurrentTimeRoundedToMinutes(value) => {
                RuntimeSubstate::CurrentTimeRoundedToMinutes(value)
            }
            PersistedSubstate::AccessRulesChain(value) => RuntimeSubstate::AccessRulesChain(value),
            PersistedSubstate::Metadata(value) => RuntimeSubstate::Metadata(value),
            PersistedSubstate::ResourceManager(value) => RuntimeSubstate::ResourceManager(value),
            PersistedSubstate::ComponentInfo(value) => RuntimeSubstate::ComponentInfo(value),
            PersistedSubstate::ComponentState(value) => RuntimeSubstate::ComponentState(value),
            PersistedSubstate::ComponentRoyaltyConfig(value) => {
                RuntimeSubstate::ComponentRoyaltyConfig(value)
            }
            PersistedSubstate::ComponentRoyaltyAccumulator(value) => {
                RuntimeSubstate::ComponentRoyaltyAccumulator(value)
            }
            PersistedSubstate::PackageInfo(value) => RuntimeSubstate::PackageInfo(value),
            PersistedSubstate::WasmCode(value) => RuntimeSubstate::WasmCode(value),
            PersistedSubstate::NativePackageInfo(value) => RuntimeSubstate::NativeCode(value),
            PersistedSubstate::PackageRoyaltyConfig(value) => {
                RuntimeSubstate::PackageRoyaltyConfig(value)
            }
            PersistedSubstate::PackageRoyaltyAccumulator(value) => {
                RuntimeSubstate::PackageRoyaltyAccumulator(value)
            }
            PersistedSubstate::PackageAccessRules(value) => {
                RuntimeSubstate::PackageAccessRules(value)
            }
            PersistedSubstate::Vault(value) => {
                RuntimeSubstate::Vault(VaultRuntimeSubstate::new(value.0))
            }
            PersistedSubstate::NonFungible(value) => RuntimeSubstate::NonFungible(value),
            PersistedSubstate::KeyValueStoreEntry(value) => {
                RuntimeSubstate::KeyValueStoreEntry(value)
            }
            PersistedSubstate::Account(value) => RuntimeSubstate::Account(value),
            PersistedSubstate::AccessController(value) => RuntimeSubstate::AccessController(value),
        }
    }
}

pub enum PersistError {
    VaultLocked,
}

#[derive(Debug)]
pub enum RuntimeSubstate {
    Global(GlobalSubstate),
    TypeInfo(TypeInfoSubstate),
    EpochManager(EpochManagerSubstate),
    ValidatorSet(ValidatorSetSubstate),
    Validator(ValidatorSubstate),
    CurrentTimeRoundedToMinutes(CurrentTimeRoundedToMinutesSubstate),
    ResourceManager(ResourceManagerSubstate),
    AccessRulesChain(ObjectAccessRulesChainSubstate),
    Metadata(MetadataSubstate),
    ComponentInfo(ComponentInfoSubstate),
    ComponentState(ComponentStateSubstate),
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(ComponentRoyaltyAccumulatorSubstate),
    NativeCode(NativeCodeSubstate),
    WasmCode(WasmCodeSubstate),
    PackageInfo(PackageInfoSubstate),
    PackageRoyaltyConfig(PackageRoyaltyConfigSubstate),
    PackageRoyaltyAccumulator(PackageRoyaltyAccumulatorSubstate),
    PackageAccessRules(PackageAccessRulesSubstate),
    Vault(VaultRuntimeSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
    AuthZoneStack(AuthZoneStackSubstate),
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    Worktop(WorktopSubstate),
    Logger(LoggerSubstate),
    TransactionRuntime(TransactionRuntimeSubstate),
    Account(AccountSubstate),
    AccessController(AccessControllerSubstate),
}

impl RuntimeSubstate {
    pub fn clone_to_persisted(&self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::Global(value) => PersistedSubstate::Global(value.clone()),
            RuntimeSubstate::TypeInfo(value) => PersistedSubstate::TypeInfo(value.clone()),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value.clone()),
            RuntimeSubstate::ValidatorSet(value) => PersistedSubstate::ValidatorSet(value.clone()),
            RuntimeSubstate::Validator(value) => PersistedSubstate::Validator(value.clone()),
            RuntimeSubstate::AccessRulesChain(value) => {
                PersistedSubstate::AccessRulesChain(value.clone())
            }
            RuntimeSubstate::CurrentTimeRoundedToMinutes(value) => {
                PersistedSubstate::CurrentTimeRoundedToMinutes(value.clone())
            }
            RuntimeSubstate::Metadata(value) => PersistedSubstate::Metadata(value.clone()),
            RuntimeSubstate::ResourceManager(value) => {
                PersistedSubstate::ResourceManager(value.clone())
            }
            RuntimeSubstate::ComponentInfo(value) => {
                PersistedSubstate::ComponentInfo(value.clone())
            }
            RuntimeSubstate::ComponentState(value) => {
                PersistedSubstate::ComponentState(value.clone())
            }
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                PersistedSubstate::ComponentRoyaltyConfig(value.clone())
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                PersistedSubstate::ComponentRoyaltyAccumulator(value.clone())
            }
            RuntimeSubstate::PackageInfo(value) => PersistedSubstate::PackageInfo(value.clone()),
            RuntimeSubstate::WasmCode(value) => PersistedSubstate::WasmCode(value.clone()),
            RuntimeSubstate::NativeCode(value) => {
                PersistedSubstate::NativePackageInfo(value.clone())
            }
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                PersistedSubstate::PackageRoyaltyConfig(value.clone())
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                PersistedSubstate::PackageRoyaltyAccumulator(value.clone())
            }
            RuntimeSubstate::PackageAccessRules(value) => {
                PersistedSubstate::PackageAccessRules(value.clone())
            }
            RuntimeSubstate::NonFungible(value) => PersistedSubstate::NonFungible(value.clone()),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                PersistedSubstate::KeyValueStoreEntry(value.clone())
            }
            RuntimeSubstate::Vault(value) => {
                let persisted_vault = value.clone_to_persisted();
                PersistedSubstate::Vault(persisted_vault)
            }
            RuntimeSubstate::Account(value) => PersistedSubstate::Account(value.clone()),
            RuntimeSubstate::AccessController(value) => {
                PersistedSubstate::AccessController(value.clone())
            }
            RuntimeSubstate::AuthZoneStack(..)
            | RuntimeSubstate::Bucket(..)
            | RuntimeSubstate::Proof(..)
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
            RuntimeSubstate::TypeInfo(value) => PersistedSubstate::TypeInfo(value),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value),
            RuntimeSubstate::ValidatorSet(value) => PersistedSubstate::ValidatorSet(value),
            RuntimeSubstate::Validator(value) => PersistedSubstate::Validator(value),
            RuntimeSubstate::AccessRulesChain(value) => PersistedSubstate::AccessRulesChain(value),
            RuntimeSubstate::CurrentTimeRoundedToMinutes(value) => {
                PersistedSubstate::CurrentTimeRoundedToMinutes(value)
            }
            RuntimeSubstate::Metadata(value) => PersistedSubstate::Metadata(value),
            RuntimeSubstate::ResourceManager(value) => PersistedSubstate::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => PersistedSubstate::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => PersistedSubstate::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                PersistedSubstate::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                PersistedSubstate::ComponentRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageInfo(value) => PersistedSubstate::PackageInfo(value),
            RuntimeSubstate::WasmCode(value) => PersistedSubstate::WasmCode(value),
            RuntimeSubstate::NativeCode(value) => PersistedSubstate::NativePackageInfo(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                PersistedSubstate::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                PersistedSubstate::PackageRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageAccessRules(value) => {
                PersistedSubstate::PackageAccessRules(value)
            }
            RuntimeSubstate::NonFungible(value) => PersistedSubstate::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                PersistedSubstate::KeyValueStoreEntry(value)
            }
            RuntimeSubstate::Vault(value) => {
                let persisted_vault = value
                    .to_persisted()
                    .expect("Vault should be liquid at end of successful transaction");
                PersistedSubstate::Vault(persisted_vault)
            }
            RuntimeSubstate::Account(value) => PersistedSubstate::Account(value),
            RuntimeSubstate::AccessController(value) => PersistedSubstate::AccessController(value),
            RuntimeSubstate::AuthZoneStack(..)
            | RuntimeSubstate::Bucket(..)
            | RuntimeSubstate::Proof(..)
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
            RuntimeSubstate::TypeInfo(value) => SubstateRefMut::TypeInfo(value),
            RuntimeSubstate::EpochManager(value) => SubstateRefMut::EpochManager(value),
            RuntimeSubstate::ValidatorSet(value) => SubstateRefMut::ValidatorSet(value),
            RuntimeSubstate::Validator(value) => SubstateRefMut::Validator(value),
            RuntimeSubstate::CurrentTimeRoundedToMinutes(value) => {
                SubstateRefMut::CurrentTimeRoundedToMinutes(value)
            }
            RuntimeSubstate::AccessRulesChain(value) => SubstateRefMut::AccessRulesChain(value),
            RuntimeSubstate::Metadata(value) => SubstateRefMut::Metadata(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRefMut::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => SubstateRefMut::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRefMut::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                SubstateRefMut::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                SubstateRefMut::ComponentRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageInfo(value) => SubstateRefMut::PackageInfo(value),
            RuntimeSubstate::WasmCode(value) => SubstateRefMut::WasmCode(value),
            RuntimeSubstate::NativeCode(value) => SubstateRefMut::NativePackageInfo(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                SubstateRefMut::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                SubstateRefMut::PackageRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageAccessRules(value) => SubstateRefMut::PackageAccessRules(value),
            RuntimeSubstate::Vault(value) => SubstateRefMut::Vault(value),
            RuntimeSubstate::NonFungible(value) => SubstateRefMut::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRefMut::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZoneStack(value) => SubstateRefMut::AuthZoneStack(value),
            RuntimeSubstate::Bucket(value) => SubstateRefMut::Bucket(value),
            RuntimeSubstate::Proof(value) => SubstateRefMut::Proof(value),
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
            RuntimeSubstate::ComponentInfo(value) => SubstateRef::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRef::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                SubstateRef::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::ComponentRoyaltyAccumulator(value) => {
                SubstateRef::ComponentRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageInfo(value) => SubstateRef::PackageInfo(value),
            RuntimeSubstate::WasmCode(value) => SubstateRef::WasmCode(value),
            RuntimeSubstate::NativeCode(value) => SubstateRef::NativeCode(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                SubstateRef::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::PackageRoyaltyAccumulator(value) => {
                SubstateRef::PackageRoyaltyAccumulator(value)
            }
            RuntimeSubstate::PackageAccessRules(value) => SubstateRef::PackageAccessRules(value),
            RuntimeSubstate::Vault(value) => SubstateRef::Vault(value),
            RuntimeSubstate::NonFungible(value) => SubstateRef::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZoneStack(value) => SubstateRef::AuthZoneStack(value),
            RuntimeSubstate::Bucket(value) => SubstateRef::Bucket(value),
            RuntimeSubstate::Proof(value) => SubstateRef::Proof(value),
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

    pub fn vault(&self) -> &VaultRuntimeSubstate {
        if let RuntimeSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn vault_mut(&mut self) -> &mut VaultRuntimeSubstate {
        if let RuntimeSubstate::Vault(vault) = self {
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

impl Into<RuntimeSubstate> for NativeCodeSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::NativeCode(self)
    }
}

impl Into<RuntimeSubstate> for TypeInfoSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::TypeInfo(self)
    }
}

impl Into<RuntimeSubstate> for ComponentInfoSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ComponentInfo(self)
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

impl Into<RuntimeSubstate> for VaultRuntimeSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::Vault(self)
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

impl Into<RuntimeSubstate> for PackageAccessRulesSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::PackageAccessRules(self)
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

impl Into<ComponentInfoSubstate> for RuntimeSubstate {
    fn into(self) -> ComponentInfoSubstate {
        if let RuntimeSubstate::ComponentInfo(component) = self {
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

impl Into<VaultRuntimeSubstate> for RuntimeSubstate {
    fn into(self) -> VaultRuntimeSubstate {
        if let RuntimeSubstate::Vault(vault) = self {
            vault
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

impl Into<BucketSubstate> for RuntimeSubstate {
    fn into(self) -> BucketSubstate {
        if let RuntimeSubstate::Bucket(substate) = self {
            substate
        } else {
            panic!("Not a bucket");
        }
    }
}

impl Into<ProofSubstate> for RuntimeSubstate {
    fn into(self) -> ProofSubstate {
        if let RuntimeSubstate::Proof(substate) = self {
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
    Proof(&'a ProofSubstate),
    Bucket(&'a BucketSubstate),
    ComponentInfo(&'a ComponentInfoSubstate),
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
    PackageAccessRules(&'a PackageAccessRulesSubstate),
    Vault(&'a VaultRuntimeSubstate),
    ResourceManager(&'a ResourceManagerSubstate),
    EpochManager(&'a EpochManagerSubstate),
    ValidatorSet(&'a ValidatorSetSubstate),
    Validator(&'a ValidatorSubstate),
    CurrentTimeRoundedToMinutes(&'a CurrentTimeRoundedToMinutesSubstate),
    AccessRulesChain(&'a ObjectAccessRulesChainSubstate),
    Metadata(&'a MetadataSubstate),
    Global(&'a GlobalSubstate),
    TypeInfo(&'a TypeInfoSubstate),
    TransactionRuntime(&'a TransactionRuntimeSubstate),
    Account(&'a AccountSubstate),
    AccessController(&'a AccessControllerSubstate),
}

impl<'a> From<SubstateRef<'a>> for &'a AuthZoneStackSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::AuthZoneStack(value) => value,
            _ => panic!("Not an auth zone stack"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a NonFungibleSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::NonFungible(value) => value,
            _ => panic!("Not a non fungible"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a EpochManagerSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::EpochManager(value) => value,
            _ => panic!("Not an epoch manager"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ValidatorSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Validator(value) => value,
            _ => panic!("Not a validator"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ComponentStateSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::ComponentState(value) => value,
            _ => panic!("Not a component state"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ComponentInfoSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::ComponentInfo(value) => value,
            _ => panic!("Not a component info"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ComponentRoyaltyConfigSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::ComponentRoyaltyConfig(value) => value,
            _ => panic!("Not a component royalty config"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ComponentRoyaltyAccumulatorSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::ComponentRoyaltyAccumulator(value) => value,
            _ => panic!("Not a component royalty accumulator"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a PackageRoyaltyConfigSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::PackageRoyaltyConfig(value) => value,
            _ => panic!("Not a packge royalty config"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a PackageRoyaltyAccumulatorSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::PackageRoyaltyAccumulator(value) => value,
            _ => panic!("Not a packge royalty accumulator"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a PackageAccessRulesSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::PackageAccessRules(value) => value,
            _ => panic!("Not a package access rules"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a WorktopSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Worktop(value) => value,
            _ => panic!("Not a worktop"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a BucketSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Bucket(value) => value,
            _ => panic!("Not a bucket"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ProofSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Proof(value) => value,
            _ => panic!("Not a proof"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a VaultRuntimeSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Vault(value) => value,
            _ => panic!("Not a vault"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a KeyValueStoreEntrySubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::KeyValueStoreEntry(value) => value,
            _ => panic!("Not a kv entry"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ResourceManagerSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::ResourceManager(value) => value,
            _ => panic!("Not a resource manager"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a TypeInfoSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::TypeInfo(value) => value,
            _ => panic!("Not a type info"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a NativeCodeSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::NativeCode(value) => value,
            _ => panic!("Not a native code"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a WasmCodeSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::WasmCode(value) => value,
            _ => panic!("Not wasm code"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a PackageInfoSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::PackageInfo(value) => value,
            _ => panic!("Not package info"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a ObjectAccessRulesChainSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::AccessRulesChain(value) => value,
            _ => panic!("Not access rules chain"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a GlobalSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Global(value) => value,
            _ => panic!("Not global"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a MetadataSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Metadata(value) => value,
            _ => panic!("Not global"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a TransactionRuntimeSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::TransactionRuntime(value) => value,
            _ => panic!("Not transaction runtime"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a CurrentTimeRoundedToMinutesSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::CurrentTimeRoundedToMinutes(value) => value,
            _ => panic!("Not current time"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a AccountSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::Account(value) => value,
            _ => panic!("Not an account"),
        }
    }
}

impl<'a> From<SubstateRef<'a>> for &'a AccessControllerSubstate {
    fn from(value: SubstateRef<'a>) -> Self {
        match value {
            SubstateRef::AccessController(value) => value,
            _ => panic!("Not an access controller"),
        }
    }
}

impl<'a> SubstateRef<'a> {
    pub fn to_scrypto_value(&self) -> IndexedScryptoValue {
        match self {
            SubstateRef::Global(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::TypeInfo(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::EpochManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::CurrentTimeRoundedToMinutes(value) => {
                IndexedScryptoValue::from_typed(*value)
            }
            SubstateRef::ResourceManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentInfo(value) => IndexedScryptoValue::from_typed(*value),
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
            SubstateRef::Vault(vault) => {
                let mut references = HashSet::new();
                references.insert(RENodeId::Global(Address::Resource(
                    vault.resource_address(),
                )));
                (references, Vec::new())
            }
            SubstateRef::Proof(proof) => {
                let mut references = HashSet::new();
                references.insert(RENodeId::Global(Address::Resource(
                    proof.resource_address(),
                )));
                (references, Vec::new())
            }
            SubstateRef::Bucket(bucket) => {
                let mut references = HashSet::new();
                references.insert(RENodeId::Global(Address::Resource(
                    bucket.resource_address(),
                )));
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
            SubstateRef::ComponentInfo(substate) => {
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
    ComponentInfo(&'a mut ComponentInfoSubstate),
    ComponentState(&'a mut ComponentStateSubstate),
    ComponentRoyaltyConfig(&'a mut ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(&'a mut ComponentRoyaltyAccumulatorSubstate),
    PackageInfo(&'a mut PackageInfoSubstate),
    WasmCode(&'a mut WasmCodeSubstate),
    NativePackageInfo(&'a mut NativeCodeSubstate),
    PackageRoyaltyConfig(&'a mut PackageRoyaltyConfigSubstate),
    PackageRoyaltyAccumulator(&'a mut PackageRoyaltyAccumulatorSubstate),
    PackageAccessRules(&'a mut PackageAccessRulesSubstate),
    NonFungible(&'a mut NonFungibleSubstate),
    KeyValueStoreEntry(&'a mut KeyValueStoreEntrySubstate),
    Vault(&'a mut VaultRuntimeSubstate),
    ResourceManager(&'a mut ResourceManagerSubstate),
    EpochManager(&'a mut EpochManagerSubstate),
    ValidatorSet(&'a mut ValidatorSetSubstate),
    Validator(&'a mut ValidatorSubstate),
    CurrentTimeRoundedToMinutes(&'a mut CurrentTimeRoundedToMinutesSubstate),
    AccessRulesChain(&'a mut ObjectAccessRulesChainSubstate),
    Metadata(&'a mut MetadataSubstate),
    Global(&'a mut GlobalSubstate),
    TypeInfo(&'a mut TypeInfoSubstate),
    Bucket(&'a mut BucketSubstate),
    Proof(&'a mut ProofSubstate),
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

    pub fn vault(&mut self) -> &mut VaultRuntimeSubstate {
        match self {
            SubstateRefMut::Vault(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn proof(&mut self) -> &mut ProofSubstate {
        match self {
            SubstateRefMut::Proof(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn bucket(&mut self) -> &mut BucketSubstate {
        match self {
            SubstateRefMut::Bucket(value) => *value,
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

    pub fn component_info(&mut self) -> &mut ComponentInfoSubstate {
        match self {
            SubstateRefMut::ComponentInfo(value) => *value,
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
