use crate::engine::*;
use crate::model::*;
use crate::types::*;

#[derive(Debug)]
pub enum RENode {
    Global(GlobalAddressSubstate), // TODO: Remove
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    AuthZone(AuthZoneStackSubstate),
    Vault(VaultRuntimeSubstate),
    Component(ComponentInfoSubstate, ComponentStateSubstate),
    Worktop(WorktopSubstate),
    Package(PackageSubstate),
    KeyValueStore(KeyValueStore),
    NonFungibleStore(NonFungibleStore),
    ResourceManager(ResourceManagerSubstate),
    System(SystemSubstate),
}

impl RENode {
    pub fn get_offsets(&self) -> Vec<(SubstateOffset, SubstateRef)> {
        match self {
            RENode::Global(substate) => {
                vec![(SubstateOffset::Global(GlobalOffset::Global), SubstateRef::Global(substate))]
            }
            RENode::Component(info, state) => {
                vec![
                    (SubstateOffset::Component(ComponentOffset::State), SubstateRef::ComponentState(state)),
                    (SubstateOffset::Component(ComponentOffset::Info), SubstateRef::ComponentInfo(info)),
                ]
            }
            RENode::KeyValueStore(store) => store
                .loaded_entries
                .iter()
                .map(|(key, value)| {
                    (SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key.clone())), SubstateRef::KeyValueStoreEntry(value))
                })
                .collect(),
            RENode::NonFungibleStore(..) => vec![],
            RENode::ResourceManager(state) => {
                vec![(SubstateOffset::ResourceManager(
                    ResourceManagerOffset::ResourceManager,
                ), SubstateRef::ResourceManager(state))]
            }
            RENode::Package(substate) => vec![
                (SubstateOffset::Package(PackageOffset::Package), SubstateRef::Package(substate))
            ],
            RENode::Bucket(substate) => vec![
                (SubstateOffset::Bucket(BucketOffset::Bucket), SubstateRef::Bucket(substate))
            ],
            RENode::Proof(proof) => vec![
                (SubstateOffset::Proof(ProofOffset::Proof), SubstateRef::Proof(proof))
            ],
            RENode::AuthZone(state) => vec![
                (SubstateOffset::AuthZone(AuthZoneOffset::AuthZone), SubstateRef::AuthZone(state))
            ],
            RENode::Vault(state) => vec![
                (SubstateOffset::Vault(VaultOffset::Vault), SubstateRef::Vault(state))
            ],
            RENode::Worktop(state) => vec![
                (SubstateOffset::Worktop(WorktopOffset::Worktop), SubstateRef::Worktop(state))
            ],
            RENode::System(state) => vec![
                (SubstateOffset::System(SystemOffset::System), SubstateRef::System(state))
            ],
        }
    }

    pub fn to_substates(self) -> HashMap<SubstateOffset, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();

        match self {
            RENode::Bucket(_) => panic!("Unexpected"),
            RENode::Proof(_) => panic!("Unexpected"),
            RENode::AuthZone(_) => panic!("Unexpected"),
            RENode::Global(global_node) => {
                substates.insert(
                    SubstateOffset::Global(GlobalOffset::Global),
                    RuntimeSubstate::GlobalRENode(global_node),
                );
            }
            RENode::Vault(vault) => {
                substates.insert(SubstateOffset::Vault(VaultOffset::Vault), vault.into());
            }
            RENode::KeyValueStore(store) => {
                for (k, v) in store.loaded_entries {
                    substates.insert(
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(k)),
                        v.into(),
                    );
                }
            }
            RENode::Component(info, state) => {
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::Info),
                    info.into(),
                );
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::State),
                    state.into(),
                );
            }
            RENode::Worktop(_) => panic!("Unexpected"),
            RENode::Package(package) => {
                substates.insert(
                    SubstateOffset::Package(PackageOffset::Package),
                    package.into(),
                );
            }
            RENode::ResourceManager(resource_manager) => {
                substates.insert(
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
                    resource_manager.into(),
                );
            }
            RENode::NonFungibleStore(non_fungible_store) => {
                for (id, non_fungible) in non_fungible_store.loaded_non_fungibles {
                    substates.insert(
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
                        non_fungible.into(),
                    );
                }
            }
            RENode::System(system) => {
                substates.insert(SubstateOffset::System(SystemOffset::System), system.into());
            }
        }

        substates
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
            RENode::NonFungibleStore(..) => Err(DropFailure::NonFungibleStore),
            RENode::Component(..) => Err(DropFailure::Component),
            RENode::Bucket(..) => Err(DropFailure::Bucket),
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
    pub fn try_drop(self) -> Result<(), DropFailure> {
        self.root.try_drop()
    }

    pub fn borrow_substate(
        &mut self,
        offset: &SubstateOffset,
    ) -> Result<SubstateRef, RuntimeError> {
        let substate_ref = match (&mut self.root, offset) {
            (
                RENode::Component(_info, state),
                SubstateOffset::Component(ComponentOffset::State),
            ) => SubstateRef::ComponentState(state),
            (RENode::Component(info, ..), SubstateOffset::Component(ComponentOffset::Info)) => {
                SubstateRef::ComponentInfo(info)
            }
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
            ) => SubstateRef::ResourceManager(resource_manager),
            (RENode::Bucket(bucket), SubstateOffset::Bucket(BucketOffset::Bucket)) => {
                SubstateRef::Bucket(bucket)
            }
            (RENode::Proof(proof), SubstateOffset::Proof(ProofOffset::Proof)) => {
                SubstateRef::Proof(proof)
            }
            (RENode::Worktop(worktop), SubstateOffset::Worktop(WorktopOffset::Worktop)) => {
                SubstateRef::Worktop(worktop)
            }
            (RENode::AuthZone(auth_zone), SubstateOffset::AuthZone(AuthZoneOffset::AuthZone)) => {
                SubstateRef::AuthZone(auth_zone)
            }
            (RENode::Vault(vault), SubstateOffset::Vault(VaultOffset::Vault)) => {
                SubstateRef::Vault(vault)
            }
            (RENode::Package(package), SubstateOffset::Package(PackageOffset::Package)) => {
                SubstateRef::Package(package)
            }
            (RENode::System(system), SubstateOffset::System(SystemOffset::System)) => {
                SubstateRef::System(system)
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
        let substate_ref = match (&mut self.root, offset) {
            (
                RENode::Component(_info, state),
                SubstateOffset::Component(ComponentOffset::State),
            ) => RawSubstateRefMut::ComponentState(state),
            (RENode::Component(info, ..), SubstateOffset::Component(ComponentOffset::Info)) => {
                RawSubstateRefMut::ComponentInfo(info)
            }
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
            ) => RawSubstateRefMut::ResourceManager(resource_manager),
            (RENode::Bucket(bucket), SubstateOffset::Bucket(BucketOffset::Bucket)) => {
                RawSubstateRefMut::Bucket(bucket)
            }
            (RENode::Proof(proof), SubstateOffset::Proof(ProofOffset::Proof)) => {
                RawSubstateRefMut::Proof(proof)
            }
            (RENode::Worktop(worktop), SubstateOffset::Worktop(WorktopOffset::Worktop)) => {
                RawSubstateRefMut::Worktop(worktop)
            }
            (RENode::AuthZone(auth_zone), SubstateOffset::AuthZone(AuthZoneOffset::AuthZone)) => {
                RawSubstateRefMut::AuthZone(auth_zone)
            }
            (RENode::Vault(vault), SubstateOffset::Vault(VaultOffset::Vault)) => {
                RawSubstateRefMut::Vault(vault)
            }
            (RENode::Package(package), SubstateOffset::Package(PackageOffset::Package)) => {
                RawSubstateRefMut::Package(package)
            }
            (RENode::System(system), SubstateOffset::System(SystemOffset::System)) => {
                RawSubstateRefMut::System(system)
            }
            (_, offset) => {
                return Err(RuntimeError::KernelError(KernelError::InvalidOffset(
                    offset.clone(),
                )));
            }
        };
        Ok(substate_ref)
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
