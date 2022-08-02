use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::model::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Substate {
    System(System),
    Resource(ResourceManager),
    Component(Component),
    ComponentState(ComponentState),
    Package(ValidatedPackage),
    Vault(Vault),
    NonFungible(NonFungibleWrapper),
    KeyValueStoreEntry(KeyValueStoreEntryWrapper),
}

impl Substate {
    pub fn vault_mut(&mut self) -> &mut Vault {
        if let Substate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn vault(&self) -> &Vault {
        if let Substate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        if let Substate::Resource(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }

    pub fn system(&self) -> &System {
        if let Substate::System(system) = self {
            system
        } else {
            panic!("Not a system value");
        }
    }

    pub fn system_mut(&mut self) -> &mut System {
        if let Substate::System(system) = self {
            system
        } else {
            panic!("Not a system value");
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        if let Substate::Resource(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }

    pub fn component_state(&self) -> &ComponentState {
        if let Substate::ComponentState(state) = self {
            state
        } else {
            panic!("Not component state");
        }
    }

    pub fn component(&self) -> &Component {
        if let Substate::Component(component) = self {
            component
        } else {
            match self {
                Substate::ComponentState(component_state) => {
                    let value = decode_any(component_state.state()).unwrap();
                    panic!("Not a component {:?}", value);
                }
                _ => {}
            }
            panic!("Not a component");
        }
    }

    pub fn component_mut(&mut self) -> &mut Component {
        if let Substate::Component(component) = self {
            component
        } else {
            panic!("Not a component");
        }
    }

    pub fn package(&self) -> &ValidatedPackage {
        if let Substate::Package(package) = self {
            package
        } else {
            panic!("Not a package");
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleWrapper {
        if let Substate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a NonFungible");
        }
    }

    pub fn kv_entry(&self) -> &KeyValueStoreEntryWrapper {
        if let Substate::KeyValueStoreEntry(kv_entry) = self {
            kv_entry
        } else {
            panic!("Not a KVEntry");
        }
    }
}

impl Into<Substate> for System {
    fn into(self) -> Substate {
        Substate::System(self)
    }
}

impl Into<Substate> for ValidatedPackage {
    fn into(self) -> Substate {
        Substate::Package(self)
    }
}

impl Into<Substate> for Component {
    fn into(self) -> Substate {
        Substate::Component(self)
    }
}

impl Into<Substate> for ComponentState {
    fn into(self) -> Substate {
        Substate::ComponentState(self)
    }
}

impl Into<Substate> for ResourceManager {
    fn into(self) -> Substate {
        Substate::Resource(self)
    }
}

impl Into<Substate> for Vault {
    fn into(self) -> Substate {
        Substate::Vault(self)
    }
}

impl Into<Substate> for NonFungibleWrapper {
    fn into(self) -> Substate {
        Substate::NonFungible(self)
    }
}

impl Into<Substate> for KeyValueStoreEntryWrapper {
    fn into(self) -> Substate {
        Substate::KeyValueStoreEntry(self)
    }
}

impl Into<Component> for Substate {
    fn into(self) -> Component {
        if let Substate::Component(component) = self {
            component
        } else {
            panic!("Not a component");
        }
    }
}

impl Into<ComponentState> for Substate {
    fn into(self) -> ComponentState {
        if let Substate::ComponentState(component_state) = self {
            component_state
        } else {
            panic!("Not a component");
        }
    }
}

impl Into<ResourceManager> for Substate {
    fn into(self) -> ResourceManager {
        if let Substate::Resource(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<ValidatedPackage> for Substate {
    fn into(self) -> ValidatedPackage {
        if let Substate::Package(package) = self {
            package
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<NonFungibleWrapper> for Substate {
    fn into(self) -> NonFungibleWrapper {
        if let Substate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a non-fungible wrapper");
        }
    }
}

impl Into<KeyValueStoreEntryWrapper> for Substate {
    fn into(self) -> KeyValueStoreEntryWrapper {
        if let Substate::KeyValueStoreEntry(kv_entry) = self {
            kv_entry
        } else {
            panic!("Not a key value store entry wrapper");
        }
    }
}

impl Into<Vault> for Substate {
    fn into(self) -> Vault {
        if let Substate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

#[derive(Debug)]
pub enum HeapRENode {
    Bucket(Bucket),
    Proof(Proof),
    Vault(Vault),
    KeyValueStore(HeapKeyValueStore),
    Component(Component, ComponentState),
    Worktop(Worktop),
    Package(ValidatedPackage),
    Resource(ResourceManager, Option<HashMap<NonFungibleId, NonFungible>>),
    System(System),
}

impl Into<HeapRENode> for (ResourceManager, Option<HashMap<NonFungibleId, NonFungible>>) {
    fn into(self) -> HeapRENode {
        HeapRENode::Resource(self.0, self.1)
    }
}

impl Into<HeapRENode> for Bucket {
    fn into(self) -> HeapRENode {
        HeapRENode::Bucket(self)
    }
}

impl Into<HeapRENode> for Proof {
    fn into(self) -> HeapRENode {
        HeapRENode::Proof(self)
    }
}

impl Into<HeapRENode> for Vault {
    fn into(self) -> HeapRENode {
        HeapRENode::Vault(self)
    }
}

impl Into<HeapRENode> for Worktop {
    fn into(self) -> HeapRENode {
        HeapRENode::Worktop(self)
    }
}

impl Into<HeapRENode> for HeapKeyValueStore {
    fn into(self) -> HeapRENode {
        HeapRENode::KeyValueStore(self)
    }
}

impl Into<HeapRENode> for ValidatedPackage {
    fn into(self) -> HeapRENode {
        HeapRENode::Package(self)
    }
}

impl Into<HeapRENode> for (Component, ComponentState) {
    fn into(self) -> HeapRENode {
        HeapRENode::Component(self.0, self.1)
    }
}

impl HeapRENode {
    pub fn get_child_nodes(&self) -> Result<HashSet<RENodeId>, RuntimeError> {
        match self {
            HeapRENode::Component(_, component_state) => {
                let value = ScryptoValue::from_slice(component_state.state())
                    .map_err(RuntimeError::DecodeError)?;
                Ok(value.node_ids())
            }
            HeapRENode::Resource(..) => Ok(HashSet::new()),
            HeapRENode::Package(..) => Ok(HashSet::new()),
            HeapRENode::Bucket(..) => Ok(HashSet::new()),
            HeapRENode::Proof(..) => Ok(HashSet::new()),
            HeapRENode::KeyValueStore(kv_store) => {
                let mut child_nodes = HashSet::new();
                for (_id, value) in &kv_store.store {
                    child_nodes.extend(value.node_ids());
                }
                Ok(child_nodes)
            }
            HeapRENode::Vault(..) => Ok(HashSet::new()),
            HeapRENode::Worktop(..) => Ok(HashSet::new()),
            HeapRENode::System(..) => Ok(HashSet::new()),
        }
    }

    pub fn system(&self) -> &System {
        match self {
            HeapRENode::System(system) => system,
            _ => panic!("Expected to be system"),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        match self {
            HeapRENode::Resource(resource_manager, ..) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        match self {
            HeapRENode::Resource(resource_manager, ..) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn non_fungibles(&self) -> &HashMap<NonFungibleId, NonFungible> {
        match self {
            HeapRENode::Resource(_, non_fungibles) => non_fungibles.as_ref().unwrap(),
            _ => panic!("Expected to be non fungibles"),
        }
    }

    pub fn non_fungibles_mut(&mut self) -> &mut HashMap<NonFungibleId, NonFungible> {
        match self {
            HeapRENode::Resource(_, non_fungibles) => non_fungibles.as_mut().unwrap(),
            _ => panic!("Expected to be non fungibles"),
        }
    }

    pub fn package(&self) -> &ValidatedPackage {
        match self {
            HeapRENode::Package(package) => package,
            _ => panic!("Expected to be a package"),
        }
    }

    pub fn component(&self) -> &Component {
        match self {
            HeapRENode::Component(component, ..) => component,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn component_mut(&mut self) -> &mut Component {
        match self {
            HeapRENode::Component(component, ..) => component,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn component_state(&self) -> &ComponentState {
        match self {
            HeapRENode::Component(_, component_state) => component_state,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn component_state_mut(&mut self) -> &mut ComponentState {
        match self {
            HeapRENode::Component(_, component_state) => component_state,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store(&self) -> &HeapKeyValueStore {
        match self {
            HeapRENode::KeyValueStore(store) => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store_mut(&mut self) -> &mut HeapKeyValueStore {
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

    pub fn verify_can_move(&self) -> Result<(), RuntimeError> {
        match self {
            HeapRENode::Bucket(bucket) => {
                if bucket.is_locked() {
                    Err(RuntimeError::CantMoveLockedBucket)
                } else {
                    Ok(())
                }
            }
            HeapRENode::Proof(proof) => {
                if proof.is_restricted() {
                    Err(RuntimeError::CantMoveRestrictedProof)
                } else {
                    Ok(())
                }
            }
            HeapRENode::KeyValueStore(..) => Ok(()),
            HeapRENode::Component(..) => Ok(()),
            HeapRENode::Vault(..) => Ok(()),
            HeapRENode::Resource(..) => Ok(()),
            HeapRENode::Package(..) => Ok(()),
            HeapRENode::Worktop(..) => Ok(()),
            HeapRENode::System(..) => Ok(()),
        }
    }

    pub fn verify_can_persist(&self) -> Result<(), RuntimeError> {
        match self {
            HeapRENode::KeyValueStore { .. } => Ok(()),
            HeapRENode::Component { .. } => Ok(()),
            HeapRENode::Vault(..) => Ok(()),
            HeapRENode::Resource(..) => Err(RuntimeError::ValueNotAllowed),
            HeapRENode::Package(..) => Err(RuntimeError::ValueNotAllowed),
            HeapRENode::Bucket(..) => Err(RuntimeError::ValueNotAllowed),
            HeapRENode::Proof(..) => Err(RuntimeError::ValueNotAllowed),
            HeapRENode::Worktop(..) => Err(RuntimeError::ValueNotAllowed),
            HeapRENode::System(..) => Err(RuntimeError::ValueNotAllowed),
        }
    }

    pub fn try_drop(self) -> Result<(), DropFailure> {
        match self {
            HeapRENode::Package(..) => Err(DropFailure::Package),
            HeapRENode::Vault(..) => Err(DropFailure::Vault),
            HeapRENode::KeyValueStore(..) => Err(DropFailure::KeyValueStore),
            HeapRENode::Component(..) => Err(DropFailure::Component),
            HeapRENode::Bucket(..) => Err(DropFailure::Bucket),
            HeapRENode::Resource(..) => Err(DropFailure::Resource),
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
