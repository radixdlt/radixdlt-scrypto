use crate::engine::*;
use crate::model::*;
use crate::types::*;

#[derive(Debug)]
pub enum RENode {
    Global(GlobalAddressSubstate),
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
    pub fn to_substates(self) -> HashMap<SubstateOffset, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();
        match self {
            RENode::Bucket(bucket) => {
                substates.insert(
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                    RuntimeSubstate::Bucket(bucket),
                );
            }
            RENode::Proof(proof) => {
                substates.insert(
                    SubstateOffset::Proof(ProofOffset::Proof),
                    RuntimeSubstate::Proof(proof),
                );
            }
            RENode::AuthZone(auth_zone) => {
                substates.insert(
                    SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
                    RuntimeSubstate::AuthZone(auth_zone),
                );
            }
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
            RENode::Worktop(worktop) => {
                substates.insert(
                    SubstateOffset::Worktop(WorktopOffset::Worktop),
                    RuntimeSubstate::Worktop(worktop),
                );
            }
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
}

#[derive(Debug)]
pub struct HeapRENode {
    pub substates: HashMap<SubstateOffset, RuntimeSubstate>,
    pub child_nodes: HashSet<RENodeId>,
}

impl HeapRENode {
    pub fn try_drop(self) -> Result<(), DropFailure> {
        // TODO: Remove this
        if !self.child_nodes.is_empty() {
            return Err(DropFailure::DroppingNodeWithChildren);
        }

        for (_, substate) in self.substates {
            match substate {
                RuntimeSubstate::AuthZone(mut auth_zone) => {
                    auth_zone.clear_all();
                    Ok(())
                }
                RuntimeSubstate::GlobalRENode(..) => panic!("Should never get here"),
                RuntimeSubstate::Package(..) => Err(DropFailure::Package),
                RuntimeSubstate::Vault(..) => Err(DropFailure::Vault),
                RuntimeSubstate::KeyValueStoreEntry(..) => Err(DropFailure::KeyValueStore),
                RuntimeSubstate::NonFungible(..) => Err(DropFailure::NonFungibleStore),
                RuntimeSubstate::ComponentInfo(..) => Err(DropFailure::Component),
                RuntimeSubstate::ComponentState(..) => Err(DropFailure::Component),
                RuntimeSubstate::Bucket(..) => Err(DropFailure::Bucket),
                RuntimeSubstate::ResourceManager(..) => Err(DropFailure::Resource),
                RuntimeSubstate::System(..) => Err(DropFailure::System),
                RuntimeSubstate::Proof(proof) => {
                    proof.drop();
                    Ok(())
                }
                RuntimeSubstate::Worktop(worktop) => worktop.drop(),
            }?;
        }

        Ok(())
    }
}

impl Into<BucketSubstate> for HeapRENode {
    fn into(mut self) -> BucketSubstate {
        self.substates
            .remove(&SubstateOffset::Bucket(BucketOffset::Bucket))
            .unwrap()
            .into()
    }
}

impl Into<ProofSubstate> for HeapRENode {
    fn into(mut self) -> ProofSubstate {
        self.substates
            .remove(&SubstateOffset::Proof(ProofOffset::Proof))
            .unwrap()
            .into()
    }
}
