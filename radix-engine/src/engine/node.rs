use crate::engine::*;
use crate::model::*;
use crate::types::*;

#[derive(Debug)]
pub enum RENode {
    Global(GlobalRENode), // TODO: Remove
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    AuthZone(AuthZoneStackSubstate),
    Vault(VaultRuntimeSubstate),
    Component(Component),
    Worktop(WorktopSubstate),
    Package(Package),
    KeyValueStore(KeyValueStore),
    NonFungibleStore(NonFungibleStore),
    ResourceManager(ResourceManager),
    System(System),
}

impl RENode {
    pub fn get_substates(&self) -> Vec<SubstateOffset> {
        match self {
            RENode::Global(..) => {
                vec![SubstateOffset::Global(GlobalOffset::Global)]
            }
            RENode::Component(..) => {
                vec![
                    SubstateOffset::Component(ComponentOffset::State),
                    SubstateOffset::Component(ComponentOffset::Info),
                ]
            }
            RENode::KeyValueStore(store) => store
                .loaded_entries
                .iter()
                .map(|(key, _)| {
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key.clone()))
                })
                .collect(),
            RENode::NonFungibleStore(..) => vec![],
            RENode::ResourceManager(..) => {
                vec![SubstateOffset::ResourceManager(
                    ResourceManagerOffset::ResourceManager,
                )]
            }
            RENode::Package(..) => vec![SubstateOffset::Package(PackageOffset::Package)],
            RENode::Bucket(..) => vec![SubstateOffset::Bucket(BucketOffset::Bucket)],
            RENode::Proof(..) => vec![SubstateOffset::Proof(ProofOffset::Proof)],
            RENode::AuthZone(..) => vec![SubstateOffset::AuthZone(AuthZoneOffset::AuthZone)],
            RENode::Vault(..) => vec![SubstateOffset::Vault(VaultOffset::Vault)],
            RENode::Worktop(..) => vec![SubstateOffset::Worktop(WorktopOffset::Worktop)],
            RENode::System(..) => vec![SubstateOffset::System(SystemOffset::System)],
        }
    }

    pub fn borrow_substate(
        &mut self,
        offset: &SubstateOffset,
    ) -> Result<SubstateRef, RuntimeError> {
        let substate_ref = match (self, offset) {
            (
                RENode::Component(component),
                SubstateOffset::Component(ComponentOffset::State),
            ) => SubstateRef::ComponentState(component.state.as_ref().unwrap()),
            (
                RENode::Component(component),
                SubstateOffset::Component(ComponentOffset::Info),
            ) => SubstateRef::ComponentInfo(&component.info),
            (
                RENode::NonFungibleStore(non_fungible_store),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
            ) => {
                let entry = non_fungible_store
                    .loaded_non_fungibles
                    .entry(id.clone())
                    .or_insert(NonFungibleSubstate(None));
                SubstateRef::NonFungible(entry)
            }
            (
                RENode::KeyValueStore(kv_store),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) => {
                let entry = kv_store
                    .loaded_entries
                    .entry(key.to_vec())
                    .or_insert(KeyValueStoreEntrySubstate(None));
                SubstateRef::KeyValueStoreEntry(entry)
            }
            (
                RENode::ResourceManager(resource_manager),
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            ) => SubstateRef::ResourceManager(&resource_manager.info),
            (RENode::Bucket(bucket), SubstateOffset::Bucket(BucketOffset::Bucket)) => {
                SubstateRef::Bucket(bucket)
            }
            (RENode::Proof(proof), SubstateOffset::Proof(ProofOffset::Proof)) => {
                SubstateRef::Proof(proof)
            }
            (RENode::Worktop(worktop), SubstateOffset::Worktop(WorktopOffset::Worktop)) => {
                SubstateRef::Worktop(worktop)
            }
            (
                RENode::AuthZone(auth_zone),
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            ) => SubstateRef::AuthZone(auth_zone),
            (RENode::Vault(vault), SubstateOffset::Vault(VaultOffset::Vault)) => {
                SubstateRef::Vault(vault)
            }
            (RENode::Package(package), SubstateOffset::Package(PackageOffset::Package)) => {
                SubstateRef::Package(&package.info)
            }
            (RENode::System(system), SubstateOffset::System(SystemOffset::System)) => {
                SubstateRef::System(&system.info)
            }
            (_, offset) => {
                return Err(RuntimeError::KernelError(KernelError::InvalidOffset(
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
                RENode::Component(component),
                SubstateOffset::Component(ComponentOffset::State),
            ) => RawSubstateRefMut::ComponentState(component.state.as_mut().unwrap()),
            (
                RENode::Component(component),
                SubstateOffset::Component(ComponentOffset::Info),
            ) => RawSubstateRefMut::ComponentInfo(&mut component.info),
            (
                RENode::NonFungibleStore(non_fungible_store),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
            ) => {
                let entry = non_fungible_store
                    .loaded_non_fungibles
                    .entry(id.clone())
                    .or_insert(NonFungibleSubstate(None));
                RawSubstateRefMut::NonFungible(entry)
            }
            (
                RENode::KeyValueStore(kv_store),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) => {
                let entry = kv_store
                    .loaded_entries
                    .entry(key.to_vec())
                    .or_insert(KeyValueStoreEntrySubstate(None));
                RawSubstateRefMut::KeyValueStoreEntry(entry)
            }
            (
                RENode::ResourceManager(resource_manager),
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            ) => RawSubstateRefMut::ResourceManager(&mut resource_manager.info),
            (RENode::Bucket(bucket), SubstateOffset::Bucket(BucketOffset::Bucket)) => {
                RawSubstateRefMut::Bucket(bucket)
            }
            (RENode::Proof(proof), SubstateOffset::Proof(ProofOffset::Proof)) => {
                RawSubstateRefMut::Proof(proof)
            }
            (RENode::Worktop(worktop), SubstateOffset::Worktop(WorktopOffset::Worktop)) => {
                RawSubstateRefMut::Worktop(worktop)
            }
            (
                RENode::AuthZone(auth_zone),
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            ) => RawSubstateRefMut::AuthZone(auth_zone),
            (RENode::Vault(vault), SubstateOffset::Vault(VaultOffset::Vault)) => {
                RawSubstateRefMut::Vault(vault)
            }
            (RENode::Package(package), SubstateOffset::Package(PackageOffset::Package)) => {
                RawSubstateRefMut::Package(&mut package.info)
            }
            (RENode::System(system), SubstateOffset::System(SystemOffset::System)) => {
                RawSubstateRefMut::System(&mut system.info)
            }
            (_, offset) => {
                return Err(RuntimeError::KernelError(KernelError::InvalidOffset(
                    offset.clone(),
                )));
            }
        };
        Ok(substate_ref)
    }

    pub fn try_drop(self) -> Result<(), DropFailure> {
        match self {
            RENode::Global(..) => panic!("Should never get here"),
            RENode::AuthZone(mut auth_zone) => {
                auth_zone.clear_all();
                Ok(())
            }
            RENode::Package(..) => Err(DropFailure::Package),
            RENode::Vault(..) => Err(DropFailure::Vault),
            RENode::KeyValueStore(..) => Err(DropFailure::KeyValueStore),
            RENode::NonFungibleStore(..) => {
                Err(DropFailure::NonFungibleStore)
            },
            RENode::Component(..) => Err(DropFailure::Component),
            RENode::Bucket(bucket) => {
                // FIXME: Hack to allow virtual buckets to be cleaned up, better would
                // FIXME: be to let auth module burn these buckets
                if bucket.resource_address().eq(&ECDSA_SECP256K1_TOKEN) {
                    Ok(())
                } else if bucket.resource_address().eq(&EDDSA_ED25519_TOKEN) {
                    Ok(())
                } else if bucket.resource_address().eq(&SYSTEM_TOKEN) {
                    Ok(())
                } else {
                    Err(DropFailure::Bucket)
                }
            }
            RENode::ResourceManager(..) => Err(DropFailure::Resource),
            RENode::System(..) => Err(DropFailure::System),
            RENode::Proof(proof) => {
                proof.drop();
                Ok(())
            }
            RENode::Worktop(worktop) => worktop.drop(),
        }
    }

    pub fn drop_nodes(nodes: Vec<HeapRENode>) -> Result<(), DropFailure> {
        let mut worktops = Vec::new();
        for node in nodes {
            // TODO: Remove this
            if !node.child_nodes.is_empty() {
                return Err(DropFailure::DroppingNodeWithChildren);
            }

            if let RENode::Worktop(worktop) = node.root {
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
pub struct HeapRENode {
    pub root: RENode,
    pub child_nodes: HashSet<RENodeId>,
}

impl HeapRENode {
    pub fn get_mut(&mut self) -> &mut RENode {
        &mut self.root
    }

    pub fn try_drop(self) -> Result<(), DropFailure> {
        self.root.try_drop()
    }
}

impl Into<BucketSubstate> for HeapRENode {
    fn into(self) -> BucketSubstate {
        match self.root {
            RENode::Bucket(bucket) => bucket,
            _ => panic!("Expected to be a bucket"),
        }
    }
}

impl Into<ProofSubstate> for HeapRENode {
    fn into(self) -> ProofSubstate {
        match self.root {
            RENode::Proof(proof) => proof,
            _ => panic!("Expected to be a proof"),
        }
    }
}
