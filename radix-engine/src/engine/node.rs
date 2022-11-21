use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::types::{
    AuthZoneOffset, BucketOffset, ComponentOffset, EpochManagerOffset, GlobalOffset,
    KeyValueStoreOffset, NonFungibleStoreOffset, PackageOffset, ProofOffset, ResourceManagerOffset,
    SubstateOffset, VaultOffset, WorktopOffset,
};

#[derive(Debug)]
pub enum RENode {
    Global(
        GlobalAddressSubstate,
    ),
    Bucket(BucketSubstate),
    Proof(ProofSubstate),
    AuthZone(AuthZoneStackSubstate),
    Vault(VaultRuntimeSubstate),
    Component(
        ComponentInfoSubstate,
        ComponentStateSubstate,
        AccessRulesSubstate,
    ),
    Worktop(WorktopSubstate),
    Package(
        PackageSubstate,
        MetadataSubstate,
    ),
    KeyValueStore(KeyValueStore),
    NonFungibleStore(NonFungibleStore),
    ResourceManager(ResourceManagerSubstate),
    EpochManager(EpochManagerSubstate),
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
                    RuntimeSubstate::Global(global_node),
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
            RENode::Component(info, state, access_rules) => {
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::Info),
                    info.into(),
                );
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::State),
                    state.into(),
                );
                substates.insert(
                    SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                    access_rules.into(),
                );

            }
            RENode::Worktop(worktop) => {
                substates.insert(
                    SubstateOffset::Worktop(WorktopOffset::Worktop),
                    RuntimeSubstate::Worktop(worktop),
                );
            }
            RENode::Package(package, metadata) => {
                substates.insert(
                    SubstateOffset::Package(PackageOffset::Package),
                    package.into(),
                );
                substates.insert(
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                    metadata.into(),
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
            RENode::EpochManager(system) => {
                substates.insert(
                    SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
                    system.into(),
                );
            }
        }

        substates
    }
}
