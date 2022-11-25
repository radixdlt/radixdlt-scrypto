use crate::engine::{KernelError, RuntimeError};
use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::types::{
    ComponentOffset, GlobalAddress, KeyValueStoreOffset, NonFungibleStoreOffset, RENodeId,
    SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum PersistedSubstate {
    Global(GlobalAddressSubstate),
    EpochManager(EpochManagerSubstate),
    ResourceManager(ResourceManagerSubstate),
    AccessRules(AccessRulesSubstate),
    ComponentInfo(ComponentInfoSubstate),
    ComponentState(ComponentStateSubstate),
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    Package(PackageSubstate),
    PackageRoyaltyConfig(PackageRoyaltyConfigSubstate),
    Vault(VaultSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
}

impl PersistedSubstate {
    pub fn vault(&self) -> &VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
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
            PersistedSubstate::EpochManager(value) => RuntimeSubstate::EpochManager(value),
            PersistedSubstate::AccessRules(value) => RuntimeSubstate::AccessRules(value),
            PersistedSubstate::ResourceManager(value) => RuntimeSubstate::ResourceManager(value),
            PersistedSubstate::ComponentInfo(value) => RuntimeSubstate::ComponentInfo(value),
            PersistedSubstate::ComponentState(value) => RuntimeSubstate::ComponentState(value),
            PersistedSubstate::ComponentRoyaltyConfig(value) => {
                RuntimeSubstate::ComponentRoyaltyConfig(value)
            }
            PersistedSubstate::Package(value) => RuntimeSubstate::Package(value),
            PersistedSubstate::PackageRoyaltyConfig(value) => {
                RuntimeSubstate::PackageRoyaltyConfig(value)
            }
            PersistedSubstate::Vault(value) => {
                RuntimeSubstate::Vault(VaultRuntimeSubstate::new(value.0))
            }
            PersistedSubstate::NonFungible(value) => RuntimeSubstate::NonFungible(value),
            PersistedSubstate::KeyValueStoreEntry(value) => {
                RuntimeSubstate::KeyValueStoreEntry(value)
            }
        }
    }
}

pub enum PersistError {
    VaultLocked,
}

#[derive(Debug)]
pub enum RuntimeSubstate {
    Global(GlobalAddressSubstate),
    EpochManager(EpochManagerSubstate),
    ResourceManager(ResourceManagerSubstate),
    AccessRules(AccessRulesSubstate),
    ComponentInfo(ComponentInfoSubstate),
    ComponentState(ComponentStateSubstate),
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    Package(PackageSubstate),
    PackageRoyaltyConfig(PackageRoyaltyConfigSubstate),
    Vault(VaultRuntimeSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
    AuthZone(AuthZoneStackSubstate),
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    Worktop(WorktopSubstate),
    FeeReserve(FeeReserveSubstate),
}

impl RuntimeSubstate {
    pub fn clone_to_persisted(&self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::Global(value) => PersistedSubstate::Global(value.clone()),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value.clone()),
            RuntimeSubstate::AccessRules(value) => PersistedSubstate::AccessRules(value.clone()),
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
            RuntimeSubstate::Package(value) => PersistedSubstate::Package(value.clone()),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                PersistedSubstate::PackageRoyaltyConfig(value.clone())
            }
            RuntimeSubstate::NonFungible(value) => PersistedSubstate::NonFungible(value.clone()),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                PersistedSubstate::KeyValueStoreEntry(value.clone())
            }
            RuntimeSubstate::Vault(value) => {
                let persisted_vault = value.clone_to_persisted();
                PersistedSubstate::Vault(persisted_vault)
            }
            RuntimeSubstate::AuthZone(..)
            | RuntimeSubstate::Bucket(..)
            | RuntimeSubstate::Proof(..)
            | RuntimeSubstate::Worktop(..)
            | RuntimeSubstate::FeeReserve(..) => {
                panic!("Should not get here");
            }
        }
    }

    pub fn to_persisted(self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::Global(value) => PersistedSubstate::Global(value),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value),
            RuntimeSubstate::AccessRules(value) => PersistedSubstate::AccessRules(value),
            RuntimeSubstate::ResourceManager(value) => PersistedSubstate::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => PersistedSubstate::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => PersistedSubstate::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                PersistedSubstate::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::Package(value) => PersistedSubstate::Package(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                PersistedSubstate::PackageRoyaltyConfig(value)
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
            RuntimeSubstate::AuthZone(..)
            | RuntimeSubstate::Bucket(..)
            | RuntimeSubstate::Proof(..)
            | RuntimeSubstate::Worktop(..)
            | RuntimeSubstate::FeeReserve(..) => {
                panic!("Should not get here");
            }
        }
    }

    pub fn decode_from_buffer(
        offset: &SubstateOffset,
        buffer: &[u8],
    ) -> Result<Self, RuntimeError> {
        let substate = match offset {
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate =
                    scrypto_decode(buffer).map_err(|e| KernelError::InvalidSborValue(e))?;
                RuntimeSubstate::ComponentState(substate)
            }
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate =
                    scrypto_decode(buffer).map_err(|e| KernelError::InvalidSborValue(e))?;
                RuntimeSubstate::KeyValueStoreEntry(substate)
            }
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)) => {
                let substate =
                    scrypto_decode(buffer).map_err(|e| KernelError::InvalidSborValue(e))?;
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
            RuntimeSubstate::EpochManager(value) => SubstateRefMut::EpochManager(value),
            RuntimeSubstate::AccessRules(value) => SubstateRefMut::AccessRules(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRefMut::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => SubstateRefMut::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRefMut::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                SubstateRefMut::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::Package(value) => SubstateRefMut::Package(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                SubstateRefMut::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::Vault(value) => SubstateRefMut::Vault(value),
            RuntimeSubstate::NonFungible(value) => SubstateRefMut::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRefMut::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZone(value) => SubstateRefMut::AuthZone(value),
            RuntimeSubstate::Bucket(value) => SubstateRefMut::Bucket(value),
            RuntimeSubstate::Proof(value) => SubstateRefMut::Proof(value),
            RuntimeSubstate::Worktop(value) => SubstateRefMut::Worktop(value),
            RuntimeSubstate::FeeReserve(value) => SubstateRefMut::FeeReserve(value),
        }
    }

    pub fn to_ref(&self) -> SubstateRef {
        match self {
            RuntimeSubstate::Global(value) => SubstateRef::Global(value),
            RuntimeSubstate::EpochManager(value) => SubstateRef::EpochManager(value),
            RuntimeSubstate::AccessRules(value) => SubstateRef::AccessRules(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRef::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => SubstateRef::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRef::ComponentState(value),
            RuntimeSubstate::ComponentRoyaltyConfig(value) => {
                SubstateRef::ComponentRoyaltyConfig(value)
            }
            RuntimeSubstate::Package(value) => SubstateRef::Package(value),
            RuntimeSubstate::PackageRoyaltyConfig(value) => {
                SubstateRef::PackageRoyaltyConfig(value)
            }
            RuntimeSubstate::Vault(value) => SubstateRef::Vault(value),
            RuntimeSubstate::NonFungible(value) => SubstateRef::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZone(value) => SubstateRef::AuthZone(value),
            RuntimeSubstate::Bucket(value) => SubstateRef::Bucket(value),
            RuntimeSubstate::Proof(value) => SubstateRef::Proof(value),
            RuntimeSubstate::Worktop(value) => SubstateRef::Worktop(value),
            RuntimeSubstate::FeeReserve(value) => SubstateRef::FeeReserve(value),
        }
    }

    pub fn global(&self) -> &GlobalAddressSubstate {
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

    pub fn package(&self) -> &PackageSubstate {
        if let RuntimeSubstate::Package(package) = self {
            package
        } else {
            panic!("Not a package");
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
}

impl Into<RuntimeSubstate> for AccessRulesSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::AccessRules(self)
    }
}

impl Into<RuntimeSubstate> for EpochManagerSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::EpochManager(self)
    }
}

impl Into<RuntimeSubstate> for PackageSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::Package(self)
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

impl Into<RuntimeSubstate> for FeeReserveSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::FeeReserve(self)
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

impl Into<PackageRoyaltyConfigSubstate> for RuntimeSubstate {
    fn into(self) -> PackageRoyaltyConfigSubstate {
        if let RuntimeSubstate::PackageRoyaltyConfig(config) = self {
            config
        } else {
            panic!("Not a package royalty config");
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

impl Into<PackageSubstate> for RuntimeSubstate {
    fn into(self) -> PackageSubstate {
        if let RuntimeSubstate::Package(package) = self {
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

impl Into<GlobalAddressSubstate> for RuntimeSubstate {
    fn into(self) -> GlobalAddressSubstate {
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

impl Into<AccessRulesSubstate> for RuntimeSubstate {
    fn into(self) -> AccessRulesSubstate {
        if let RuntimeSubstate::AccessRules(substate) = self {
            substate
        } else {
            panic!("Not access rules");
        }
    }
}

pub enum SubstateRef<'a> {
    AuthZone(&'a AuthZoneStackSubstate),
    Worktop(&'a WorktopSubstate),
    FeeReserve(&'a FeeReserveSubstate),
    Proof(&'a ProofSubstate),
    Bucket(&'a BucketSubstate),
    ComponentInfo(&'a ComponentInfoSubstate),
    ComponentState(&'a ComponentStateSubstate),
    ComponentRoyaltyConfig(&'a ComponentRoyaltyConfigSubstate),
    NonFungible(&'a NonFungibleSubstate),
    KeyValueStoreEntry(&'a KeyValueStoreEntrySubstate),
    Package(&'a PackageSubstate),
    PackageRoyaltyConfig(&'a PackageRoyaltyConfigSubstate),
    Vault(&'a VaultRuntimeSubstate),
    ResourceManager(&'a ResourceManagerSubstate),
    EpochManager(&'a EpochManagerSubstate),
    AccessRules(&'a AccessRulesSubstate),
    Global(&'a GlobalAddressSubstate),
}

impl<'a> SubstateRef<'a> {
    pub fn to_scrypto_value(&self) -> IndexedScryptoValue {
        match self {
            SubstateRef::Global(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::EpochManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ResourceManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentInfo(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentState(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentRoyaltyConfig(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::Package(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::PackageRoyaltyConfig(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::NonFungible(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::KeyValueStoreEntry(value) => IndexedScryptoValue::from_typed(*value),
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
            SubstateRef::EpochManager(system) => *system,
            _ => panic!("Not a system substate"),
        }
    }

    pub fn component_state(&self) -> &ComponentStateSubstate {
        match self {
            SubstateRef::ComponentState(state) => *state,
            _ => panic!("Not a component state"),
        }
    }

    pub fn component_info(&self) -> &ComponentInfoSubstate {
        match self {
            SubstateRef::ComponentInfo(info) => *info,
            _ => panic!("Not a component info"),
        }
    }

    pub fn component_royalty_config(&self) -> &ComponentRoyaltyConfigSubstate {
        match self {
            SubstateRef::ComponentRoyaltyConfig(info) => *info,
            _ => panic!("Not a component royalty config"),
        }
    }

    pub fn package_royalty_config(&self) -> &PackageRoyaltyConfigSubstate {
        match self {
            SubstateRef::PackageRoyaltyConfig(info) => *info,
            _ => panic!("Not a package royalty config"),
        }
    }

    pub fn proof(&self) -> &ProofSubstate {
        match self {
            SubstateRef::Proof(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn auth_zone(&self) -> &AuthZoneStackSubstate {
        match self {
            SubstateRef::AuthZone(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&self) -> &WorktopSubstate {
        match self {
            SubstateRef::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn fee_reserve(&self) -> &FeeReserveSubstate {
        match self {
            SubstateRef::FeeReserve(value) => *value,
            _ => panic!("Not a fee reserve"),
        }
    }

    pub fn bucket(&self) -> &BucketSubstate {
        match self {
            SubstateRef::Bucket(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn vault(&self) -> &VaultRuntimeSubstate {
        match self {
            SubstateRef::Vault(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManagerSubstate {
        match self {
            SubstateRef::ResourceManager(value) => *value,
            _ => panic!("Not a resource manager"),
        }
    }

    pub fn package(&self) -> &PackageSubstate {
        match self {
            SubstateRef::Package(value) => *value,
            _ => panic!("Not a package"),
        }
    }

    pub fn access_rules(&self) -> &AccessRulesSubstate {
        match self {
            SubstateRef::AccessRules(value) => *value,
            _ => panic!("Not access rules"),
        }
    }

    pub fn global_address(&self) -> &GlobalAddressSubstate {
        match self {
            SubstateRef::Global(value) => *value,
            _ => panic!("Not a global address"),
        }
    }

    pub fn references_and_owned_nodes(&self) -> (HashSet<GlobalAddress>, HashSet<RENodeId>) {
        match self {
            SubstateRef::Global(global) => {
                let mut owned_nodes = HashSet::new();
                match global {
                    GlobalAddressSubstate::Resource(resource_manager_id) => {
                        owned_nodes.insert(RENodeId::ResourceManager(*resource_manager_id))
                    }
                    GlobalAddressSubstate::Component(component_id) => {
                        owned_nodes.insert(RENodeId::Component(*component_id))
                    }
                    GlobalAddressSubstate::System(epoch_manager_id) => {
                        owned_nodes.insert(RENodeId::EpochManager(*epoch_manager_id))
                    }
                    GlobalAddressSubstate::Package(package_id) => {
                        owned_nodes.insert(RENodeId::Package(*package_id))
                    }
                };

                (HashSet::new(), owned_nodes)
            }
            SubstateRef::Vault(vault) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Resource(vault.resource_address()));
                (references, HashSet::new())
            }
            SubstateRef::Proof(proof) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Resource(proof.resource_address()));
                (references, HashSet::new())
            }
            SubstateRef::Bucket(bucket) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Resource(bucket.resource_address()));
                (references, HashSet::new())
            }
            SubstateRef::ComponentInfo(substate) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Package(substate.package_address));
                (references, HashSet::new())
            }
            SubstateRef::ResourceManager(substate) => {
                let mut owned_nodes = HashSet::new();
                if let Some(nf_store_id) = substate.nf_store_id {
                    owned_nodes.insert(RENodeId::NonFungibleStore(nf_store_id));
                }
                (HashSet::new(), owned_nodes)
            }
            SubstateRef::ComponentState(substate) => {
                let scrypto_value = IndexedScryptoValue::from_slice(&substate.raw).unwrap();
                (scrypto_value.global_references(), scrypto_value.node_ids())
            }
            SubstateRef::KeyValueStoreEntry(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|raw| IndexedScryptoValue::from_slice(raw).unwrap());
                if let Some(scrypto_value) = maybe_scrypto_value {
                    (scrypto_value.global_references(), scrypto_value.node_ids())
                } else {
                    (HashSet::new(), HashSet::new())
                }
            }
            SubstateRef::NonFungible(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|non_fungible| IndexedScryptoValue::from_typed(non_fungible));
                if let Some(scrypto_value) = maybe_scrypto_value {
                    (scrypto_value.global_references(), scrypto_value.node_ids())
                } else {
                    (HashSet::new(), HashSet::new())
                }
            }
            _ => (HashSet::new(), HashSet::new()),
        }
    }
}

pub enum SubstateRefMut<'a> {
    ComponentInfo(&'a mut ComponentInfoSubstate),
    ComponentState(&'a mut ComponentStateSubstate),
    ComponentRoyaltyConfig(&'a mut ComponentRoyaltyConfigSubstate),
    Package(&'a mut PackageSubstate),
    PackageRoyaltyConfig(&'a mut PackageRoyaltyConfigSubstate),
    NonFungible(&'a mut NonFungibleSubstate),
    KeyValueStoreEntry(&'a mut KeyValueStoreEntrySubstate),
    Vault(&'a mut VaultRuntimeSubstate),
    ResourceManager(&'a mut ResourceManagerSubstate),
    EpochManager(&'a mut EpochManagerSubstate),
    AccessRules(&'a mut AccessRulesSubstate),
    Global(&'a mut GlobalAddressSubstate),
    Bucket(&'a mut BucketSubstate),
    Proof(&'a mut ProofSubstate),
    Worktop(&'a mut WorktopSubstate),
    FeeReserve(&'a mut FeeReserveSubstate),
    AuthZone(&'a mut AuthZoneStackSubstate),
}

impl<'a> SubstateRefMut<'a> {
    pub fn auth_zone(&mut self) -> &mut AuthZoneStackSubstate {
        match self {
            SubstateRefMut::AuthZone(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&mut self) -> &mut WorktopSubstate {
        match self {
            SubstateRefMut::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn fee_reserve(&mut self) -> &mut FeeReserveSubstate {
        match self {
            SubstateRefMut::FeeReserve(value) => *value,
            _ => panic!("Not a fee reserve"),
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

    pub fn package_royalty_config(&mut self) -> &mut PackageRoyaltyConfigSubstate {
        match self {
            SubstateRefMut::PackageRoyaltyConfig(value) => *value,
            _ => panic!("Not package royalty config"),
        }
    }

    pub fn epoch_manager(&mut self) -> &mut EpochManagerSubstate {
        match self {
            SubstateRefMut::EpochManager(value) => *value,
            _ => panic!("Not epoch manager"),
        }
    }

    pub fn access_rules(&mut self) -> &mut AccessRulesSubstate {
        match self {
            SubstateRefMut::AccessRules(value) => *value,
            _ => panic!("Not access rules"),
        }
    }
}
