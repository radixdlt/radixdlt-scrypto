use crate::engine::*;
use crate::model::*;
use crate::types::*;

#[derive(Debug)]
pub enum HeapRENode {
    Global(GlobalRENode), // TODO: Remove
    Bucket(Bucket),
    Proof(Proof),
    AuthZone(AuthZone),
    Vault(Vault),
    Component(Component),
    Worktop(Worktop),
    Package(Package),
    KeyValueStore(KeyValueStore),
    NonFungibleStore(NonFungibleStore),
    ResourceManager(ResourceManager),
    System(System),
}

impl HeapRENode {
    /// Not that this method is intended for heap nodes only, see the panic below.
    pub fn get_child_nodes(&self) -> Result<HashSet<RENodeId>, RuntimeError> {
        match self {
            HeapRENode::Global(global_node) => {
                let child_node = match &global_node.address {
                    GlobalAddressSubstate::Component(component) => RENodeId::Component(component.0),
                    GlobalAddressSubstate::SystemComponent(component) => {
                        RENodeId::System(component.0)
                    }
                    GlobalAddressSubstate::Package(package_address) => {
                        RENodeId::Package(*package_address)
                    }
                    GlobalAddressSubstate::Resource(resource_address) => {
                        RENodeId::ResourceManager(*resource_address)
                    }
                };
                let mut child_nodes = HashSet::new();
                child_nodes.insert(child_node);
                Ok(child_nodes)
            }
            HeapRENode::Component(component) => {
                if let Some(state) = &component.state {
                    let value = ScryptoValue::from_slice(&state.raw)
                        .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                    Ok(value.node_ids())
                } else {
                    panic!("Component state should be available for heap component")
                }
            }
            HeapRENode::KeyValueStore(store) => {
                let mut child_nodes = HashSet::new();
                for (_id, substate) in &store.loaded_entries {
                    if let Some(v) = &substate.0 {
                        let value = ScryptoValue::from_slice(&v)
                            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                        child_nodes.extend(value.node_ids());
                    }
                }
                Ok(child_nodes)
            }
            HeapRENode::NonFungibleStore(..) => Ok(HashSet::new()),
            HeapRENode::ResourceManager(resource_manager) => {
                let mut child_nodes = HashSet::new();
                if let Some(non_fungible_store_id) = &resource_manager.info.non_fungible_store_id {
                    child_nodes
                        .insert(RENodeId::NonFungibleStore(non_fungible_store_id.to_owned()));
                }
                Ok(child_nodes)
            }
            HeapRENode::Package(..) => Ok(HashSet::new()),
            HeapRENode::Bucket(..) => Ok(HashSet::new()),
            HeapRENode::Proof(..) => Ok(HashSet::new()),
            HeapRENode::AuthZone(..) => Ok(HashSet::new()),
            HeapRENode::Vault(..) => Ok(HashSet::new()),
            HeapRENode::Worktop(..) => Ok(HashSet::new()),
            HeapRENode::System(..) => Ok(HashSet::new()),
        }
    }

    pub fn borrow_substate(
        &mut self,
        offset: &SubstateOffset,
    ) -> Result<SubstateRef, RuntimeError> {
        let substate_ref = match (self, offset) {
            (
                HeapRENode::Component(component),
                SubstateOffset::Component(ComponentOffset::State),
            ) => SubstateRef::ComponentState(component.state.as_ref().unwrap()),
            (
                HeapRENode::Component(component),
                SubstateOffset::Component(ComponentOffset::Info),
            ) => SubstateRef::ComponentInfo(&component.info),
            (
                HeapRENode::NonFungibleStore(non_fungible_store),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
            ) => {
                let entry = non_fungible_store
                    .loaded_non_fungibles
                    .entry(id.clone())
                    .or_insert(NonFungibleSubstate(None));
                SubstateRef::NonFungible(entry)
            }
            (
                HeapRENode::KeyValueStore(kv_store),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) => {
                let entry = kv_store
                    .loaded_entries
                    .entry(key.to_vec())
                    .or_insert(KeyValueStoreEntrySubstate(None));
                SubstateRef::KeyValueStoreEntry(entry)
            }
            (
                HeapRENode::ResourceManager(resource_manager),
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            ) => SubstateRef::ResourceManager(&resource_manager.info),
            (HeapRENode::Bucket(bucket), SubstateOffset::Bucket(BucketOffset::Bucket)) => {
                SubstateRef::Bucket(bucket)
            }
            (HeapRENode::Proof(proof), SubstateOffset::Proof(ProofOffset::Proof)) => {
                SubstateRef::Proof(proof)
            }
            (HeapRENode::Worktop(worktop), SubstateOffset::Worktop(WorktopOffset::Worktop)) => {
                SubstateRef::Worktop(worktop)
            }
            (HeapRENode::AuthZone(auth_zone), SubstateOffset::AuthZone(AuthZoneOffset::AuthZone)) => {
                SubstateRef::AuthZone(auth_zone)
            }
            (_, offset) => {
                return Err(RuntimeError::KernelError(KernelError::OffsetNotAvailable(
                    offset.clone(),
                )));
            }
        };
        Ok(substate_ref)
    }

    pub fn borrow_substate_mut(
        &mut self,
        offset: &SubstateOffset,
    ) -> Result<RawSubstateRefMut, RuntimeError> {
        let substate_ref = match (self, offset) {
            (
                HeapRENode::Component(component),
                SubstateOffset::Component(ComponentOffset::State),
            ) => RawSubstateRefMut::ComponentState(component.state.as_mut().unwrap()),
            (
                HeapRENode::Component(component),
                SubstateOffset::Component(ComponentOffset::Info),
            ) => RawSubstateRefMut::ComponentInfo(&mut component.info),
            (
                HeapRENode::NonFungibleStore(non_fungible_store),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
            ) => {
                let entry = non_fungible_store
                    .loaded_non_fungibles
                    .entry(id.clone())
                    .or_insert(NonFungibleSubstate(None));
                RawSubstateRefMut::NonFungible(entry)
            }
            (
                HeapRENode::KeyValueStore(kv_store),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) => {
                let entry = kv_store
                    .loaded_entries
                    .entry(key.to_vec())
                    .or_insert(KeyValueStoreEntrySubstate(None));
                RawSubstateRefMut::KeyValueStoreEntry(entry)
            }
            (
                HeapRENode::ResourceManager(resource_manager),
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            ) => RawSubstateRefMut::ResourceManager(&mut resource_manager.info),
            (HeapRENode::Bucket(bucket), SubstateOffset::Bucket(BucketOffset::Bucket)) => {
                RawSubstateRefMut::Bucket(bucket)
            }
            (HeapRENode::Proof(proof), SubstateOffset::Proof(ProofOffset::Proof)) => {
                RawSubstateRefMut::Proof(proof)
            }
            (HeapRENode::Worktop(worktop), SubstateOffset::Worktop(WorktopOffset::Worktop)) => {
                RawSubstateRefMut::Worktop(worktop)
            }
            (HeapRENode::AuthZone(auth_zone), SubstateOffset::AuthZone(AuthZoneOffset::AuthZone)) => {
                RawSubstateRefMut::AuthZone(auth_zone)
            }
            (_, offset) => {
                return Err(RuntimeError::KernelError(KernelError::OffsetNotAvailable(
                    offset.clone(),
                )));
            }
        };
        Ok(substate_ref)
    }

    pub fn auth_zone(&self) -> &AuthZone {
        match self {
            HeapRENode::AuthZone(auth_zone, ..) => auth_zone,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn vault(&self) -> &Vault {
        match self {
            HeapRENode::Vault(vault) => vault,
            _ => panic!("Expected to be a vault"),
        }
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        match self {
            HeapRENode::Vault(vault) => vault,
            _ => panic!("Expected to be a vault"),
        }
    }

    pub fn verify_can_move(&self) -> Result<(), RuntimeError> {
        match self {
            HeapRENode::AuthZone(..) => {
                Err(RuntimeError::KernelError(KernelError::CantMoveAuthZone))
            }
            HeapRENode::Bucket(bucket) => {
                if bucket.is_locked() {
                    Err(RuntimeError::KernelError(KernelError::CantMoveLockedBucket))
                } else {
                    Ok(())
                }
            }
            HeapRENode::Proof(proof) => {
                if proof.is_restricted() {
                    Err(RuntimeError::KernelError(
                        KernelError::CantMoveRestrictedProof,
                    ))
                } else {
                    Ok(())
                }
            }
            HeapRENode::KeyValueStore(..) => Ok(()),
            HeapRENode::NonFungibleStore(..) => Ok(()),
            HeapRENode::Component(..) => Ok(()),
            HeapRENode::Vault(..) => Ok(()),
            HeapRENode::ResourceManager(..) => Ok(()),
            HeapRENode::Package(..) => Ok(()),
            HeapRENode::Worktop(..) => Err(RuntimeError::KernelError(KernelError::CantMoveWorktop)),
            HeapRENode::System(..) => Ok(()),
            HeapRENode::Global(..) => Err(RuntimeError::KernelError(KernelError::CantMoveGlobal)),
        }
    }

    pub fn verify_can_persist(&self) -> Result<(), RuntimeError> {
        match self {
            HeapRENode::Global { .. } => Ok(()),
            HeapRENode::KeyValueStore { .. } => Ok(()),
            HeapRENode::NonFungibleStore { .. } => Ok(()),
            HeapRENode::Component { .. } => Ok(()),
            HeapRENode::Vault(..) => Ok(()),
            HeapRENode::ResourceManager(..) => {
                Err(RuntimeError::KernelError(KernelError::ValueNotAllowed))
            }
            HeapRENode::AuthZone(..) => {
                Err(RuntimeError::KernelError(KernelError::ValueNotAllowed))
            }
            HeapRENode::Package(..) => Err(RuntimeError::KernelError(KernelError::ValueNotAllowed)),
            HeapRENode::Bucket(..) => Err(RuntimeError::KernelError(KernelError::ValueNotAllowed)),
            HeapRENode::Proof(..) => Err(RuntimeError::KernelError(KernelError::ValueNotAllowed)),
            HeapRENode::Worktop(..) => Err(RuntimeError::KernelError(KernelError::ValueNotAllowed)),
            HeapRENode::System(..) => Err(RuntimeError::KernelError(KernelError::ValueNotAllowed)),
        }
    }

    pub fn try_drop(self) -> Result<(), DropFailure> {
        match self {
            HeapRENode::Global(..) => panic!("Should never get here"),
            HeapRENode::AuthZone(mut auth_zone) => {
                auth_zone.clear();
                Ok(())
            }
            HeapRENode::Package(..) => Err(DropFailure::Package),
            HeapRENode::Vault(..) => Err(DropFailure::Vault),
            HeapRENode::KeyValueStore(..) => Err(DropFailure::KeyValueStore),
            HeapRENode::NonFungibleStore(..) => Err(DropFailure::NonFungibleStore),
            HeapRENode::Component(..) => Err(DropFailure::Component),
            HeapRENode::Bucket(..) => Err(DropFailure::Bucket),
            HeapRENode::ResourceManager(..) => Err(DropFailure::Resource),
            HeapRENode::System(..) => Err(DropFailure::System),
            HeapRENode::Proof(proof) => {
                proof.drop();
                Ok(())
            }
            HeapRENode::Worktop(worktop) => worktop.drop(),
        }
    }

    pub fn drop_nodes(nodes: Vec<HeapRootRENode>) -> Result<(), DropFailure> {
        let mut worktops = Vec::new();
        for node in nodes {
            if let HeapRENode::Worktop(worktop) = node.root {
                worktops.push(worktop);
            } else {
                node.try_drop()?;
            }
        }
        for worktop in worktops {
            worktop.drop()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct HeapRootRENode {
    pub root: HeapRENode,
    pub child_nodes: HashMap<RENodeId, HeapRENode>,
}

impl HeapRootRENode {
    pub fn root(&self) -> &HeapRENode {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut HeapRENode {
        &mut self.root
    }

    pub fn non_root(&self, id: &RENodeId) -> &HeapRENode {
        self.child_nodes.get(id).unwrap()
    }

    pub fn non_root_mut(&mut self, id: &RENodeId) -> &mut HeapRENode {
        self.child_nodes.get_mut(id).unwrap()
    }

    pub fn get_node(&self, id: Option<&RENodeId>) -> &HeapRENode {
        if let Some(node_id) = id {
            self.child_nodes.get(node_id).unwrap()
        } else {
            &self.root
        }
    }

    pub fn get_node_mut(&mut self, id: Option<&RENodeId>) -> &mut HeapRENode {
        if let Some(node_id) = id {
            self.child_nodes.get_mut(node_id).unwrap()
        } else {
            &mut self.root
        }
    }

    pub fn insert_non_root_nodes(&mut self, nodes: HashMap<RENodeId, HeapRENode>) {
        for (id, node) in nodes {
            self.child_nodes.insert(id, node);
        }
    }

    pub fn to_nodes(self, root_id: RENodeId) -> HashMap<RENodeId, HeapRENode> {
        let mut nodes = self.child_nodes;
        nodes.insert(root_id, self.root);
        nodes
    }

    pub fn try_drop(self) -> Result<(), DropFailure> {
        self.root.try_drop()
    }
}

impl Into<Bucket> for HeapRootRENode {
    fn into(self) -> Bucket {
        match self.root {
            HeapRENode::Bucket(bucket) => bucket,
            _ => panic!("Expected to be a bucket"),
        }
    }
}

impl Into<Proof> for HeapRootRENode {
    fn into(self) -> Proof {
        match self.root {
            HeapRENode::Proof(proof) => proof,
            _ => panic!("Expected to be a proof"),
        }
    }
}
