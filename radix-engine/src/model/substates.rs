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
    ComponentInfo(ComponentInfoSubstate),
    AccessRules(AccessRulesSubstate),
    ComponentState(ComponentStateSubstate),
    Package(PackageSubstate),
    Vault(VaultSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
    CurrentTimeInMillis(CurrentTimeInMillisSubstate),
    CurrentTimeInSeconds(CurrentTimeInSecondsSubstate),
    CurrentTimeInMinutes(CurrentTimeInMinutesSubstate),
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
            PersistedSubstate::CurrentTimeInMillis(value) => {
                RuntimeSubstate::CurrentTimeInMillis(value)
            }
            PersistedSubstate::CurrentTimeInSeconds(value) => {
                RuntimeSubstate::CurrentTimeInSeconds(value)
            }
            PersistedSubstate::CurrentTimeInMinutes(value) => {
                RuntimeSubstate::CurrentTimeInMinutes(value)
            }
            PersistedSubstate::AccessRules(value) => RuntimeSubstate::AccessRules(value),
            PersistedSubstate::ResourceManager(value) => RuntimeSubstate::ResourceManager(value),
            PersistedSubstate::ComponentInfo(value) => RuntimeSubstate::ComponentInfo(value),
            PersistedSubstate::ComponentState(value) => RuntimeSubstate::ComponentState(value),
            PersistedSubstate::Package(value) => RuntimeSubstate::Package(value),
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
    ComponentInfo(ComponentInfoSubstate),
    AccessRules(AccessRulesSubstate),
    ComponentState(ComponentStateSubstate),
    Package(PackageSubstate),
    Vault(VaultRuntimeSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
    AuthZone(AuthZoneStackSubstate),
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    Worktop(WorktopSubstate),
    CurrentTimeInMillis(CurrentTimeInMillisSubstate),
    CurrentTimeInSeconds(CurrentTimeInSecondsSubstate),
    CurrentTimeInMinutes(CurrentTimeInMinutesSubstate),
}

impl RuntimeSubstate {
    pub fn clone_to_persisted(&self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::Global(value) => PersistedSubstate::Global(value.clone()),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value.clone()),
            RuntimeSubstate::CurrentTimeInMillis(value) => {
                PersistedSubstate::CurrentTimeInMillis(value.clone())
            }
            RuntimeSubstate::CurrentTimeInSeconds(value) => {
                PersistedSubstate::CurrentTimeInSeconds(value.clone())
            }
            RuntimeSubstate::CurrentTimeInMinutes(value) => {
                PersistedSubstate::CurrentTimeInMinutes(value.clone())
            }
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
            RuntimeSubstate::Package(value) => PersistedSubstate::Package(value.clone()),
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
            | RuntimeSubstate::Worktop(..) => {
                panic!("Should not get here");
            }
        }
    }

    pub fn to_persisted(self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::Global(value) => PersistedSubstate::Global(value),
            RuntimeSubstate::EpochManager(value) => PersistedSubstate::EpochManager(value),
            RuntimeSubstate::CurrentTimeInMillis(value) => {
                PersistedSubstate::CurrentTimeInMillis(value)
            }
            RuntimeSubstate::CurrentTimeInSeconds(value) => {
                PersistedSubstate::CurrentTimeInSeconds(value)
            }
            RuntimeSubstate::CurrentTimeInMinutes(value) => {
                PersistedSubstate::CurrentTimeInMinutes(value)
            }
            RuntimeSubstate::AccessRules(value) => PersistedSubstate::AccessRules(value),
            RuntimeSubstate::ResourceManager(value) => PersistedSubstate::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => PersistedSubstate::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => PersistedSubstate::ComponentState(value),
            RuntimeSubstate::Package(value) => PersistedSubstate::Package(value),
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
            | RuntimeSubstate::Worktop(..) => {
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
            RuntimeSubstate::CurrentTimeInMillis(value) => {
                SubstateRefMut::CurrentTimeInMillis(value)
            }
            RuntimeSubstate::CurrentTimeInSeconds(value) => {
                SubstateRefMut::CurrentTimeInSeconds(value)
            }
            RuntimeSubstate::CurrentTimeInMinutes(value) => {
                SubstateRefMut::CurrentTimeInMinutes(value)
            }
            RuntimeSubstate::AccessRules(value) => SubstateRefMut::AccessRules(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRefMut::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => SubstateRefMut::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRefMut::ComponentState(value),
            RuntimeSubstate::Package(value) => SubstateRefMut::Package(value),
            RuntimeSubstate::Vault(value) => SubstateRefMut::Vault(value),
            RuntimeSubstate::NonFungible(value) => SubstateRefMut::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRefMut::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZone(value) => SubstateRefMut::AuthZone(value),
            RuntimeSubstate::Bucket(value) => SubstateRefMut::Bucket(value),
            RuntimeSubstate::Proof(value) => SubstateRefMut::Proof(value),
            RuntimeSubstate::Worktop(value) => SubstateRefMut::Worktop(value),
        }
    }

    pub fn to_ref(&self) -> SubstateRef {
        match self {
            RuntimeSubstate::Global(value) => SubstateRef::Global(value),
            RuntimeSubstate::EpochManager(value) => SubstateRef::EpochManager(value),
            RuntimeSubstate::CurrentTimeInMillis(value) => SubstateRef::CurrentTimeInMillis(value),
            RuntimeSubstate::CurrentTimeInSeconds(value) => {
                SubstateRef::CurrentTimeInSeconds(value)
            }
            RuntimeSubstate::CurrentTimeInMinutes(value) => {
                SubstateRef::CurrentTimeInMinutes(value)
            }
            RuntimeSubstate::AccessRules(value) => SubstateRef::AccessRules(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRef::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => SubstateRef::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRef::ComponentState(value),
            RuntimeSubstate::Package(value) => SubstateRef::Package(value),
            RuntimeSubstate::Vault(value) => SubstateRef::Vault(value),
            RuntimeSubstate::NonFungible(value) => SubstateRef::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
            RuntimeSubstate::AuthZone(value) => SubstateRef::AuthZone(value),
            RuntimeSubstate::Bucket(value) => SubstateRef::Bucket(value),
            RuntimeSubstate::Proof(value) => SubstateRef::Proof(value),
            RuntimeSubstate::Worktop(value) => SubstateRef::Worktop(value),
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

impl Into<RuntimeSubstate> for CurrentTimeInMillisSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::CurrentTimeInMillis(self)
    }
}

impl Into<RuntimeSubstate> for CurrentTimeInSecondsSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::CurrentTimeInSeconds(self)
    }
}

impl Into<RuntimeSubstate> for CurrentTimeInMinutesSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::CurrentTimeInMinutes(self)
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
    Proof(&'a ProofSubstate),
    Bucket(&'a BucketSubstate),
    ComponentInfo(&'a ComponentInfoSubstate),
    ComponentState(&'a ComponentStateSubstate),
    NonFungible(&'a NonFungibleSubstate),
    KeyValueStoreEntry(&'a KeyValueStoreEntrySubstate),
    Package(&'a PackageSubstate),
    Vault(&'a VaultRuntimeSubstate),
    ResourceManager(&'a ResourceManagerSubstate),
    EpochManager(&'a EpochManagerSubstate),
    AccessRules(&'a AccessRulesSubstate),
    Global(&'a GlobalAddressSubstate),
    CurrentTimeInMillis(&'a CurrentTimeInMillisSubstate),
    CurrentTimeInSeconds(&'a CurrentTimeInSecondsSubstate),
    CurrentTimeInMinutes(&'a CurrentTimeInMinutesSubstate),
}

impl<'a> SubstateRef<'a> {
    pub fn to_scrypto_value(&self) -> IndexedScryptoValue {
        match self {
            SubstateRef::Global(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::EpochManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::CurrentTimeInMillis(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::CurrentTimeInSeconds(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::CurrentTimeInMinutes(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ResourceManager(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentInfo(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::ComponentState(value) => IndexedScryptoValue::from_typed(*value),
            SubstateRef::Package(value) => IndexedScryptoValue::from_typed(*value),
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
            SubstateRef::EpochManager(epoch_manager_substate) => *epoch_manager_substate,
            _ => panic!("Not an epoch manager substate"),
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

    pub fn current_time_in_minutes(&self) -> &CurrentTimeInMinutesSubstate {
        match self {
            SubstateRef::CurrentTimeInMinutes(substate) => *substate,
            _ => panic!("Not a current time in minutes substate"),
        }
    }

    pub fn references_and_owned_nodes(&self) -> (HashSet<GlobalAddress>, HashSet<RENodeId>) {
        match self {
            SubstateRef::Global(global) => {
                let mut owned_nodes = HashSet::new();
                match global {
                    GlobalAddressSubstate::Resource(resource_address) => {
                        owned_nodes.insert(RENodeId::ResourceManager(*resource_address))
                    }
                    GlobalAddressSubstate::Component(component) => {
                        owned_nodes.insert(RENodeId::Component(component.0))
                    }
                    GlobalAddressSubstate::System(system_id) => match system_id {
                        SystemId::EpochManager(epoch_manager_id) => {
                            owned_nodes.insert(RENodeId::EpochManager(*epoch_manager_id))
                        }
                        SystemId::Clock(clock_id) => owned_nodes.insert(RENodeId::Clock(*clock_id)),
                    },
                    GlobalAddressSubstate::Package(package_address) => {
                        owned_nodes.insert(RENodeId::Package(*package_address))
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
    NonFungible(&'a mut NonFungibleSubstate),
    KeyValueStoreEntry(&'a mut KeyValueStoreEntrySubstate),
    Package(&'a mut PackageSubstate),
    Vault(&'a mut VaultRuntimeSubstate),
    ResourceManager(&'a mut ResourceManagerSubstate),
    EpochManager(&'a mut EpochManagerSubstate),
    AccessRules(&'a mut AccessRulesSubstate),
    Global(&'a mut GlobalAddressSubstate),
    Bucket(&'a mut BucketSubstate),
    Proof(&'a mut ProofSubstate),
    Worktop(&'a mut WorktopSubstate),
    AuthZone(&'a mut AuthZoneStackSubstate),
    CurrentTimeInMillis(&'a mut CurrentTimeInMillisSubstate),
    CurrentTimeInSeconds(&'a mut CurrentTimeInSecondsSubstate),
    CurrentTimeInMinutes(&'a mut CurrentTimeInMinutesSubstate),
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
            _ => panic!("Not system"),
        }
    }

    pub fn epoch_manager(&mut self) -> &mut EpochManagerSubstate {
        match self {
            SubstateRefMut::EpochManager(value) => *value,
            _ => panic!("Not an epoch manager"),
        }
    }

    pub fn access_rules(&mut self) -> &mut AccessRulesSubstate {
        match self {
            SubstateRefMut::AccessRules(value) => *value,
            _ => panic!("Not access rules"),
        }
    }

    pub fn current_time_in_millis(&mut self) -> &mut CurrentTimeInMillisSubstate {
        match self {
            SubstateRefMut::CurrentTimeInMillis(value) => *value,
            _ => panic!("Not current time in millis"),
        }
    }

    pub fn current_time_in_seconds(&mut self) -> &mut CurrentTimeInSecondsSubstate {
        match self {
            SubstateRefMut::CurrentTimeInSeconds(value) => *value,
            _ => panic!("Not current time in seconds"),
        }
    }

    pub fn current_time_in_minutes(&mut self) -> &mut CurrentTimeInMinutesSubstate {
        match self {
            SubstateRefMut::CurrentTimeInMinutes(value) => *value,
            _ => panic!("Not current time in minutes"),
        }
    }
}
