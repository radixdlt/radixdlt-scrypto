use crate::engine::*;
use crate::model::*;
use crate::types::*;

#[derive(Debug)]
pub enum HeapRENode {
    Global(GlobalRENode), // TODO: Remove
    Bucket(Bucket),
    Proof(Proof),
    AuthZone(AuthZone),
    Vault(VaultRuntimeSubstate),
    Component(Component),
    Worktop(Worktop),
    Package(Package),
    KeyValueStore(KeyValueStore),
    NonFungibleStore(NonFungibleStore),
    ResourceManager(ResourceManager),
    System(System),
}

impl HeapRENode {
    pub fn get_substates(&self) -> Vec<SubstateOffset> {
        match self {
            HeapRENode::Global(..) => {
                vec![SubstateOffset::Global(GlobalOffset::Global)]
            }
            HeapRENode::Component(..) => {
                vec![
                    SubstateOffset::Component(ComponentOffset::State),
                    SubstateOffset::Component(ComponentOffset::Info),
                ]
            }
            HeapRENode::KeyValueStore(store) => store
                .loaded_entries
                .iter()
                .map(|(key, _)| {
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key.clone()))
                })
                .collect(),
            HeapRENode::NonFungibleStore(..) => vec![],
            HeapRENode::ResourceManager(..) => {
                vec![SubstateOffset::ResourceManager(
                    ResourceManagerOffset::ResourceManager,
                )]
            }
            HeapRENode::Package(..) => vec![SubstateOffset::Package(PackageOffset::Package)],
            HeapRENode::Bucket(..) => vec![SubstateOffset::Bucket(BucketOffset::Bucket)],
            HeapRENode::Proof(..) => vec![SubstateOffset::Proof(ProofOffset::Proof)],
            HeapRENode::AuthZone(..) => vec![SubstateOffset::AuthZone(AuthZoneOffset::AuthZone)],
            HeapRENode::Vault(..) => vec![SubstateOffset::Vault(VaultOffset::Vault)],
            HeapRENode::Worktop(..) => vec![SubstateOffset::Worktop(WorktopOffset::Worktop)],
            HeapRENode::System(..) => vec![SubstateOffset::System(SystemOffset::System)],
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
            (
                HeapRENode::AuthZone(auth_zone),
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            ) => SubstateRef::AuthZone(auth_zone),
            (HeapRENode::Vault(vault), SubstateOffset::Vault(VaultOffset::Vault)) => {
                SubstateRef::Vault(vault)
            }
            (HeapRENode::Package(package), SubstateOffset::Package(PackageOffset::Package)) => {
                SubstateRef::Package(&package.info)
            }
            (HeapRENode::System(system), SubstateOffset::System(SystemOffset::System)) => {
                SubstateRef::System(&system.info)
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
            (
                HeapRENode::AuthZone(auth_zone),
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            ) => RawSubstateRefMut::AuthZone(auth_zone),
            (HeapRENode::Vault(vault), SubstateOffset::Vault(VaultOffset::Vault)) => {
                RawSubstateRefMut::Vault(vault)
            }
            (HeapRENode::Package(package), SubstateOffset::Package(PackageOffset::Package)) => {
                RawSubstateRefMut::Package(&mut package.info)
            }
            (HeapRENode::System(system), SubstateOffset::System(SystemOffset::System)) => {
                RawSubstateRefMut::System(&mut system.info)
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
