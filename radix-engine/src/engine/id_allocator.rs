use crate::engine::{KernelError, RuntimeError};
use radix_engine_interface::address::EntityType;
use radix_engine_interface::api::types::{
    AccessControllerId, AuthZoneStackId, BucketId, ComponentId, FeeReserveId, GlobalAddress,
    KeyValueStoreId, NonFungibleStoreId, PackageId, ProofId, RENodeId, RENodeType,
    ResourceManagerId, TransactionRuntimeId, ValidatorId, VaultId,
};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::model::*;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::vec;
use sbor::rust::vec::Vec;

use crate::engine::IdAllocationError;

/// An ID allocator defines how identities are generated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdAllocator {
    pre_allocated_ids: BTreeSet<RENodeId>,
    frame_allocated_ids: Vec<BTreeSet<RENodeId>>,
    next_entity_ids: BTreeMap<EntityType, u32>,
    next_id: u32,
    transaction_hash: Hash,
}

impl IdAllocator {
    /// Creates an ID allocator.
    pub fn new(transaction_hash: Hash, pre_allocated_ids: BTreeSet<RENodeId>) -> Self {
        Self {
            pre_allocated_ids,
            frame_allocated_ids: vec![BTreeSet::new()],
            next_entity_ids: BTreeMap::new(),
            next_id: 0u32,
            transaction_hash,
        }
    }

    pub fn pre_execute_invocation(&mut self) {
        self.frame_allocated_ids.push(BTreeSet::new());
    }

    pub fn post_execute_invocation(&mut self) -> Result<(), RuntimeError> {
        let ids = self.frame_allocated_ids.pop().expect("No frame found");
        if !ids.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::IdAllocationError(
                IdAllocationError::AllocatedIDsNotEmpty,
            )));
        }
        Ok(())
    }

    pub fn take_node_id(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        let ids = self.frame_allocated_ids.last_mut().expect("No frame found");
        let frame_allocated = ids.remove(&node_id);
        let pre_allocated = self.pre_allocated_ids.remove(&node_id);
        if !frame_allocated && !pre_allocated {
            return Err(RuntimeError::KernelError(KernelError::IdAllocationError(
                IdAllocationError::RENodeIdWasNotAllocated(node_id),
            )));
        }
        Ok(())
    }

    pub fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        let node_id = match node_type {
            RENodeType::AuthZoneStack => self
                .new_auth_zone_id()
                .map(|id| RENodeId::AuthZoneStack(id)),
            RENodeType::Bucket => self.new_bucket_id().map(|id| RENodeId::Bucket(id)),
            RENodeType::Proof => self.new_proof_id().map(|id| RENodeId::Proof(id)),
            RENodeType::TransactionRuntime => self
                .new_transaction_hash_id()
                .map(|id| RENodeId::TransactionRuntime(id)),
            RENodeType::Worktop => Ok(RENodeId::Worktop),
            RENodeType::Logger => Ok(RENodeId::Logger),
            RENodeType::Vault => self.new_vault_id().map(|id| RENodeId::Vault(id)),
            RENodeType::KeyValueStore => {
                self.new_kv_store_id().map(|id| RENodeId::KeyValueStore(id))
            }
            RENodeType::NonFungibleStore => self
                .new_nf_store_id()
                .map(|id| RENodeId::NonFungibleStore(id)),
            RENodeType::Package => {
                // Security Alert: ensure ID allocating will practically never fail
                self.new_package_id().map(|id| RENodeId::Package(id))
            }
            RENodeType::ResourceManager => self
                .new_resource_manager_id()
                .map(|id| RENodeId::ResourceManager(id)),
            RENodeType::Component => self.new_component_id().map(|id| RENodeId::Component(id)),
            RENodeType::EpochManager => {
                self.new_component_id().map(|id| RENodeId::EpochManager(id))
            }
            RENodeType::Validator => self.new_validator_id().map(|id| RENodeId::Validator(id)),
            RENodeType::Clock => self.new_component_id().map(|id| RENodeId::Clock(id)),
            RENodeType::AccessController => self
                .new_access_controller_id()
                .map(|id| RENodeId::AccessController(id)),
            RENodeType::Identity => self.new_component_id().map(|id| RENodeId::Identity(id)),
            RENodeType::GlobalPackage => self
                .new_package_address()
                .map(|address| RENodeId::Global(GlobalAddress::Package(address))),
            RENodeType::GlobalEpochManager => self
                .new_epoch_manager_address()
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
            RENodeType::GlobalValidator => self
                .new_validator_address()
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
            RENodeType::GlobalClock => self
                .new_clock_address()
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
            RENodeType::GlobalResourceManager => self
                .new_resource_address()
                .map(|address| RENodeId::Global(GlobalAddress::Resource(address))),
            RENodeType::GlobalAccount => self
                .new_account_address()
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
            RENodeType::GlobalIdentity => self
                .new_identity_address()
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
            RENodeType::GlobalComponent => self
                .new_component_address()
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
            RENodeType::GlobalAccessController => self
                .new_access_controller_address()
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
        }
        .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

        let ids = self
            .frame_allocated_ids
            .last_mut()
            .expect("No frame found.");
        ids.insert(node_id);

        Ok(node_id)
    }

    fn next(&mut self) -> Result<u32, IdAllocationError> {
        if self.next_id == u32::MAX {
            Err(IdAllocationError::OutOfID)
        } else {
            let rtn = self.next_id;
            self.next_id += 1;
            Ok(rtn)
        }
    }

    fn next_id(&mut self) -> Result<[u8; 36], IdAllocationError> {
        let mut buf = [0u8; 36];
        (&mut buf[0..32]).copy_from_slice(&self.transaction_hash.0);
        (&mut buf[32..]).copy_from_slice(&self.next()?.to_le_bytes());
        Ok(buf)
    }

    fn next_entity_id(&mut self, entity_type: EntityType) -> Result<u32, IdAllocationError> {
        let rtn = if let Some(next) = self.next_entity_ids.get_mut(&entity_type) {
            let cur = *next;
            if cur == u32::MAX {
                return Err(IdAllocationError::OutOfID);
            }
            *next += 1;
            cur
        } else {
            self.next_entity_ids.insert(entity_type, 1u32);
            0u32
        };

        Ok(rtn)
    }

    /// Creates a new package ID.
    pub fn new_package_address(&mut self) -> Result<PackageAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        let next_id = self.next_entity_id(EntityType::Package)?;
        data.extend(next_id.to_le_bytes());
        Ok(PackageAddress::Normal(hash(data).lower_26_bytes()))
    }

    pub fn new_identity_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(ComponentAddress::Identity(hash(data).lower_26_bytes()))
    }

    pub fn new_account_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        let next_id = self.next_entity_id(EntityType::AccountComponent)?;
        data.extend(next_id.to_le_bytes());
        Ok(ComponentAddress::Account(hash(data).lower_26_bytes()))
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        let next_id = self.next_entity_id(EntityType::NormalComponent)?;
        data.extend(next_id.to_le_bytes());
        Ok(ComponentAddress::Normal(hash(data).lower_26_bytes()))
    }

    pub fn new_validator_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        let next_id = self.next_entity_id(EntityType::Validator)?;
        data.extend(next_id.to_le_bytes());
        Ok(ComponentAddress::Validator(hash(data).lower_26_bytes()))
    }

    pub fn new_epoch_manager_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        let next_id = self.next_entity_id(EntityType::EpochManager)?;
        data.extend(next_id.to_le_bytes());
        Ok(ComponentAddress::EpochManager(hash(data).lower_26_bytes()))
    }

    pub fn new_clock_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        let next_id = self.next_entity_id(EntityType::Clock)?;
        data.extend(next_id.to_le_bytes());
        Ok(ComponentAddress::Clock(hash(data).lower_26_bytes()))
    }

    pub fn new_access_controller_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        Ok(ComponentAddress::AccessController(
            hash(data).lower_26_bytes(),
        ))
    }

    /// Creates a new resource address.
    pub fn new_resource_address(&mut self) -> Result<ResourceAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        let next_id = self.next_entity_id(EntityType::Resource)?;
        data.extend(next_id.to_le_bytes());
        Ok(ResourceAddress::Normal(hash(data).lower_26_bytes()))
    }

    pub fn new_auth_zone_id(&mut self) -> Result<AuthZoneStackId, IdAllocationError> {
        Ok(self.next()?)
    }

    pub fn new_fee_reserve_id(&mut self) -> Result<FeeReserveId, IdAllocationError> {
        Ok(self.next()?)
    }

    /// Creates a new bucket ID.
    pub fn new_bucket_id(&mut self) -> Result<BucketId, IdAllocationError> {
        Ok(self.next()?)
    }

    /// Creates a new proof ID.
    pub fn new_proof_id(&mut self) -> Result<ProofId, IdAllocationError> {
        Ok(self.next()?)
    }

    /// Creates a new vault ID.
    pub fn new_vault_id(&mut self) -> Result<VaultId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_transaction_hash_id(&mut self) -> Result<TransactionRuntimeId, IdAllocationError> {
        self.next()
    }

    pub fn new_component_id(&mut self) -> Result<ComponentId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_validator_id(&mut self) -> Result<ValidatorId, IdAllocationError> {
        self.next_id()
    }

    /// Creates a new key value store ID.
    pub fn new_kv_store_id(&mut self) -> Result<KeyValueStoreId, IdAllocationError> {
        self.next_id()
    }

    /// Creates a new non-fungible store ID.
    pub fn new_nf_store_id(&mut self) -> Result<NonFungibleStoreId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_resource_manager_id(&mut self) -> Result<ResourceManagerId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_package_id(&mut self) -> Result<PackageId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_access_controller_id(&mut self) -> Result<AccessControllerId, IdAllocationError> {
        self.next_id()
    }
}
