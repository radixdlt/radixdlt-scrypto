use crate::engine::*;
use crate::model::*;
use crate::types::*;

pub enum SubstateRef<'a> {
    ComponentInfo(&'a ComponentInfoSubstate),
    ComponentState(&'a ComponentStateSubstate),
    NonFungible(&'a NonFungibleSubstate),
    KeyValueStoreEntry(&'a KeyValueStoreEntrySubstate),
    Package(&'a PackageSubstate),
    Vault(&'a VaultSubstate),
    ResourceManager(&'a ResourceManagerSubstate),
    System(&'a SystemSubstate),
    Global(&'a GlobalAddressSubstate),
}

impl<'a> SubstateRef<'a> {
    pub fn to_scrypto_value(&self) -> ScryptoValue {
        match self {
            SubstateRef::Global(value) => ScryptoValue::from_typed(*value),
            SubstateRef::System(value) => ScryptoValue::from_typed(*value),
            SubstateRef::ResourceManager(value) => ScryptoValue::from_typed(*value),
            SubstateRef::ComponentInfo(value) => ScryptoValue::from_typed(*value),
            SubstateRef::ComponentState(value) => ScryptoValue::from_typed(*value),
            SubstateRef::Package(value) => ScryptoValue::from_typed(*value),
            SubstateRef::Vault(value) => ScryptoValue::from_typed(*value),
            SubstateRef::NonFungible(value) => ScryptoValue::from_typed(*value),
            SubstateRef::KeyValueStoreEntry(value) => ScryptoValue::from_typed(*value),
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleSubstate {
        match self {
            SubstateRef::NonFungible(non_fungible_substate) => *non_fungible_substate,
            _ => panic!("Not a non fungible"),
        }
    }

    pub fn system(&self) -> &SystemSubstate {
        match self {
            SubstateRef::System(system) => *system,
            _ => panic!("Not a system substate"),
        }
    }

    pub fn references_and_owned_nodes(&self) -> (HashSet<GlobalAddress>, HashSet<RENodeId>) {
        match self {
            SubstateRef::ComponentState(substate) => {
                let scrypto_value = ScryptoValue::from_slice(&substate.raw).unwrap();
                (scrypto_value.global_references(), scrypto_value.node_ids())
            }
            SubstateRef::KeyValueStoreEntry(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|raw| ScryptoValue::from_slice(raw).unwrap());
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
                    .map(|non_fungible| ScryptoValue::from_typed(non_fungible));
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
            HeapRENode::ResourceManager(..) => Ok(HashSet::new()),
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
                HeapRENode::ResourceManager(resource_manager),
                SubstateOffset::ResourceManager(ResourceManagerOffset::NonFungible(id)),
            ) => {
                let entry = resource_manager
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
            (_, offset) => {
                return Err(RuntimeError::KernelError(KernelError::OffsetNotAvailable(
                    offset.clone(),
                )));
            }
        };
        Ok(substate_ref)
    }

    pub fn global_re_node(&self) -> &GlobalRENode {
        match self {
            HeapRENode::Global(global_node) => global_node,
            _ => panic!("Expected to be global node"),
        }
    }

    pub fn system(&self) -> &System {
        match self {
            HeapRENode::System(system) => system,
            _ => panic!("Expected to be system"),
        }
    }

    pub fn system_mut(&mut self) -> &mut System {
        match self {
            HeapRENode::System(system) => system,
            _ => panic!("Expected to be system"),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        match self {
            HeapRENode::ResourceManager(resource_manager, ..) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        match self {
            HeapRENode::ResourceManager(resource_manager, ..) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn auth_zone(&self) -> &AuthZone {
        match self {
            HeapRENode::AuthZone(auth_zone, ..) => auth_zone,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn auth_zone_mut(&mut self) -> &mut AuthZone {
        match self {
            HeapRENode::AuthZone(auth_zone, ..) => auth_zone,
            _ => panic!("Expected to be an auth zone"),
        }
    }

    pub fn package(&self) -> &Package {
        match self {
            HeapRENode::Package(package) => package,
            _ => panic!("Expected to be a package"),
        }
    }

    pub fn package_mut(&mut self) -> &Package {
        match self {
            HeapRENode::Package(package) => package,
            _ => panic!("Expected to be a package"),
        }
    }

    pub fn bucket(&self) -> &Bucket {
        match self {
            HeapRENode::Bucket(bucket) => bucket,
            _ => panic!("Expected to be a bucket"),
        }
    }
    pub fn bucket_mut(&mut self) -> &mut Bucket {
        match self {
            HeapRENode::Bucket(bucket) => bucket,
            _ => panic!("Expected to be a bucket"),
        }
    }

    pub fn proof(&self) -> &Proof {
        match self {
            HeapRENode::Proof(proof) => proof,
            _ => panic!("Expected to be a proof"),
        }
    }
    pub fn proof_mut(&mut self) -> &mut Proof {
        match self {
            HeapRENode::Proof(proof) => proof,
            _ => panic!("Expected to be a proof"),
        }
    }

    pub fn component(&self) -> &Component {
        match self {
            HeapRENode::Component(component, ..) => component,
            _ => panic!("Expected to be a component"),
        }
    }

    pub fn component_mut(&mut self) -> &mut Component {
        match self {
            HeapRENode::Component(component, ..) => component,
            _ => panic!("Expected to be a component"),
        }
    }

    pub fn kv_store(&self) -> &KeyValueStore {
        match self {
            HeapRENode::KeyValueStore(store) => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store_mut(&mut self) -> &mut KeyValueStore {
        match self {
            HeapRENode::KeyValueStore(store) => store,
            _ => panic!("Expected to be a store"),
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

    pub fn worktop(&self) -> &Worktop {
        match self {
            HeapRENode::Worktop(worktop) => worktop,
            _ => panic!("Expected to be a worktop"),
        }
    }

    pub fn worktop_mut(&mut self) -> &mut Worktop {
        match self {
            HeapRENode::Worktop(worktop) => worktop,
            _ => panic!("Expected to be a worktop"),
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
