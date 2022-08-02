use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::model::*;

/// Represents a Radix Engine address. Each maps a unique substate key.
///
/// TODO: separate space addresses?
///
/// FIXME: RESIM listing is broken ATM.
/// By using scrypto codec, we lose sorting capability of the address space.
/// Can also be resolved by A) using prefix search instead of range search or B) use special codec as before
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SubstateId {
    // TODO: Remove this bool which represents globalization
    ComponentInfo(ComponentAddress, bool),
    Package(PackageAddress),
    ResourceManager(ResourceAddress),
    NonFungibleSpace(ResourceAddress),
    NonFungible(ResourceAddress, NonFungibleId),
    KeyValueStoreSpace(KeyValueStoreId),
    KeyValueStoreEntry(KeyValueStoreId, Vec<u8>),
    Vault(VaultId),
    ComponentState(ComponentAddress),
    System,
}

impl SubstateId {
    pub fn get_node_id(&self) -> RENodeId {
        match self {
            SubstateId::ComponentInfo(component_address, ..) => {
                RENodeId::Component(*component_address)
            }
            SubstateId::ComponentState(component_address) => {
                RENodeId::Component(*component_address)
            }
            SubstateId::NonFungibleSpace(resource_address) => RENodeId::Resource(*resource_address),
            SubstateId::NonFungible(resource_address, ..) => RENodeId::Resource(*resource_address),
            SubstateId::KeyValueStoreSpace(kv_store_id) => RENodeId::KeyValueStore(*kv_store_id),
            SubstateId::KeyValueStoreEntry(kv_store_id, ..) => {
                RENodeId::KeyValueStore(*kv_store_id)
            }
            SubstateId::Vault(vault_id) => RENodeId::Vault(*vault_id),
            SubstateId::Package(package_address) => RENodeId::Package(*package_address),
            SubstateId::ResourceManager(resource_address) => RENodeId::Resource(*resource_address),
            SubstateId::System => RENodeId::System,
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            SubstateId::KeyValueStoreEntry(..) => false,
            SubstateId::ComponentState(..) => false,
            SubstateId::NonFungible(..) => false,
            SubstateId::ComponentInfo(..) => true,
            SubstateId::NonFungibleSpace(..) => true,
            SubstateId::KeyValueStoreSpace(..) => true,
            SubstateId::Vault(..) => true,
            SubstateId::Package(..) => true,
            SubstateId::ResourceManager(..) => true,
            SubstateId::System => true,
        }
    }

    pub fn can_own_nodes(&self) -> bool {
        match self {
            SubstateId::KeyValueStoreEntry(..) => true,
            SubstateId::ComponentState(..) => true,
            SubstateId::ComponentInfo(..) => false,
            SubstateId::NonFungible(..) => false,
            SubstateId::NonFungibleSpace(..) => false,
            SubstateId::KeyValueStoreSpace(..) => false,
            SubstateId::Vault(..) => false,
            SubstateId::Package(..) => false,
            SubstateId::ResourceManager(..) => false,
            SubstateId::System => false,
        }
    }
}

impl Into<ComponentAddress> for SubstateId {
    fn into(self) -> ComponentAddress {
        match self {
            SubstateId::ComponentInfo(component_address, ..)
            | SubstateId::ComponentState(component_address) => component_address,
            _ => panic!("Address is not a component address"),
        }
    }
}

impl Into<ResourceAddress> for SubstateId {
    fn into(self) -> ResourceAddress {
        if let SubstateId::ResourceManager(resource_address) = self {
            return resource_address;
        } else {
            panic!("Address is not a resource address");
        }
    }
}

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
pub enum RENode {
    Bucket(Bucket),
    Proof(Proof),
    Vault(Vault),
    KeyValueStore(PreCommittedKeyValueStore),
    Component(Component, ComponentState),
    Worktop(Worktop),
    Package(ValidatedPackage),
    Resource(ResourceManager, Option<HashMap<NonFungibleId, NonFungible>>),
    System(System),
}

impl RENode {
    pub fn system(&self) -> &System {
        match self {
            RENode::System(system) => system,
            _ => panic!("Expected to be system"),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        match self {
            RENode::Resource(resource_manager, ..) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        match self {
            RENode::Resource(resource_manager, ..) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn non_fungibles(&self) -> &HashMap<NonFungibleId, NonFungible> {
        match self {
            RENode::Resource(_, non_fungibles) => non_fungibles.as_ref().unwrap(),
            _ => panic!("Expected to be non fungibles"),
        }
    }

    pub fn non_fungibles_mut(&mut self) -> &mut HashMap<NonFungibleId, NonFungible> {
        match self {
            RENode::Resource(_, non_fungibles) => non_fungibles.as_mut().unwrap(),
            _ => panic!("Expected to be non fungibles"),
        }
    }

    pub fn package(&self) -> &ValidatedPackage {
        match self {
            RENode::Package(package) => package,
            _ => panic!("Expected to be a package"),
        }
    }

    pub fn component(&self) -> &Component {
        match self {
            RENode::Component(component, ..) => component,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn component_mut(&mut self) -> &mut Component {
        match self {
            RENode::Component(component, ..) => component,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn component_state(&self) -> &ComponentState {
        match self {
            RENode::Component(_, component_state) => component_state,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn component_state_mut(&mut self) -> &mut ComponentState {
        match self {
            RENode::Component(_, component_state) => component_state,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store(&self) -> &PreCommittedKeyValueStore {
        match self {
            RENode::KeyValueStore(store) => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store_mut(&mut self) -> &mut PreCommittedKeyValueStore {
        match self {
            RENode::KeyValueStore(store) => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn vault(&self) -> &Vault {
        match self {
            RENode::Vault(vault) => vault,
            _ => panic!("Expected to be a vault"),
        }
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        match self {
            RENode::Vault(vault) => vault,
            _ => panic!("Expected to be a vault"),
        }
    }

    pub fn verify_can_move(&self) -> Result<(), RuntimeError> {
        match self {
            RENode::Bucket(bucket) => {
                if bucket.is_locked() {
                    Err(RuntimeError::CantMoveLockedBucket)
                } else {
                    Ok(())
                }
            }
            RENode::Proof(proof) => {
                if proof.is_restricted() {
                    Err(RuntimeError::CantMoveRestrictedProof)
                } else {
                    Ok(())
                }
            }
            RENode::KeyValueStore(..) => Ok(()),
            RENode::Component(..) => Ok(()),
            RENode::Vault(..) => Ok(()),
            RENode::Resource(..) => Ok(()),
            RENode::Package(..) => Ok(()),
            RENode::Worktop(..) => Ok(()),
            RENode::System(..) => Ok(()),
        }
    }

    pub fn verify_can_persist(&self) -> Result<(), RuntimeError> {
        match self {
            RENode::KeyValueStore { .. } => Ok(()),
            RENode::Component { .. } => Ok(()),
            RENode::Vault(..) => Ok(()),
            RENode::Resource(..) => Err(RuntimeError::ValueNotAllowed),
            RENode::Package(..) => Err(RuntimeError::ValueNotAllowed),
            RENode::Bucket(..) => Err(RuntimeError::ValueNotAllowed),
            RENode::Proof(..) => Err(RuntimeError::ValueNotAllowed),
            RENode::Worktop(..) => Err(RuntimeError::ValueNotAllowed),
            RENode::System(..) => Err(RuntimeError::ValueNotAllowed),
        }
    }

    pub fn try_drop(self) -> Result<(), DropFailure> {
        match self {
            RENode::Package(..) => Err(DropFailure::Package),
            RENode::Vault(..) => Err(DropFailure::Vault),
            RENode::KeyValueStore(..) => Err(DropFailure::KeyValueStore),
            RENode::Component(..) => Err(DropFailure::Component),
            RENode::Bucket(..) => Err(DropFailure::Bucket),
            RENode::Resource(..) => Err(DropFailure::Resource),
            RENode::System(..) => Err(DropFailure::System),
            RENode::Proof(proof) => {
                proof.drop();
                Ok(())
            }
            RENode::Worktop(worktop) => worktop.drop(),
        }
    }

    pub fn drop_nodes(nodes: Vec<HeapRootRENode>) -> Result<(), DropFailure> {
        let mut worktops = Vec::new();
        for node in nodes {
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
pub struct HeapRootRENode {
    pub root: RENode,
    pub non_root_nodes: HashMap<RENodeId, RENode>,
}

impl HeapRootRENode {
    pub fn root(&self) -> &RENode {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut RENode {
        &mut self.root
    }

    pub fn non_root(&self, id: &RENodeId) -> &RENode {
        self.non_root_nodes.get(id).unwrap()
    }

    pub fn non_root_mut(&mut self, id: &RENodeId) -> &mut RENode {
        self.non_root_nodes.get_mut(id).unwrap()
    }

    pub fn get_node(&self, id: Option<&RENodeId>) -> &RENode {
        if let Some(node_id) = id {
            self.non_root_nodes.get(node_id).unwrap()
        } else {
            &self.root
        }
    }

    pub fn get_node_mut(&mut self, id: Option<&RENodeId>) -> &mut RENode {
        if let Some(node_id) = id {
            self.non_root_nodes.get_mut(node_id).unwrap()
        } else {
            &mut self.root
        }
    }

    pub fn insert_non_root_nodes(&mut self, nodes: HashMap<RENodeId, RENode>) {
        for (id, node) in nodes {
            self.non_root_nodes.insert(id, node);
        }
    }

    pub fn to_nodes(self, root_id: RENodeId) -> HashMap<RENodeId, RENode> {
        let mut nodes = self.non_root_nodes;
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
            RENode::Bucket(bucket) => bucket,
            _ => panic!("Expected to be a bucket"),
        }
    }
}

impl Into<Proof> for HeapRootRENode {
    fn into(self) -> Proof {
        match self.root {
            RENode::Proof(proof) => proof,
            _ => panic!("Expected to be a proof"),
        }
    }
}

#[derive(Debug)]
pub enum REComplexValue {
    Component(Component, ComponentState),
}

impl REComplexValue {
    pub fn get_children(&self) -> Result<HashSet<RENodeId>, RuntimeError> {
        match self {
            REComplexValue::Component(_, component_state) => {
                let value = ScryptoValue::from_slice(component_state.state())
                    .map_err(RuntimeError::DecodeError)?;
                Ok(value.node_ids())
            }
        }
    }

    pub fn into_re_node(self, child_nodes: HashMap<RENodeId, HeapRootRENode>) -> HeapRootRENode {
        let mut non_root_nodes = HashMap::new();
        for (id, val) in child_nodes {
            non_root_nodes.extend(val.to_nodes(id));
        }
        match self {
            REComplexValue::Component(component, component_state) => HeapRootRENode {
                root: RENode::Component(component, component_state),
                non_root_nodes,
            },
        }
    }
}

#[derive(Debug)]
pub enum REPrimitiveNode {
    Package(ValidatedPackage),
    Bucket(Bucket),
    Proof(Proof),
    KeyValue(PreCommittedKeyValueStore),
    Resource(ResourceManager, Option<HashMap<NonFungibleId, NonFungible>>),
    Vault(Vault),
    Worktop(Worktop),
}

#[derive(Debug)]
pub enum RENodeByComplexity {
    Primitive(REPrimitiveNode),
    Complex(REComplexValue),
}

impl Into<HeapRootRENode> for REPrimitiveNode {
    fn into(self) -> HeapRootRENode {
        let root = match self {
            REPrimitiveNode::Resource(resource_manager, maybe_non_fungibles) => {
                RENode::Resource(resource_manager, maybe_non_fungibles)
            }
            REPrimitiveNode::Package(package) => RENode::Package(package),
            REPrimitiveNode::Bucket(bucket) => RENode::Bucket(bucket),
            REPrimitiveNode::Proof(proof) => RENode::Proof(proof),
            REPrimitiveNode::KeyValue(store) => RENode::KeyValueStore(store),
            REPrimitiveNode::Vault(vault) => RENode::Vault(vault),

            REPrimitiveNode::Worktop(worktop) => RENode::Worktop(worktop),
        };
        HeapRootRENode {
            root,
            non_root_nodes: HashMap::new(),
        }
    }
}

impl Into<RENodeByComplexity> for (ResourceManager, Option<HashMap<NonFungibleId, NonFungible>>) {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Primitive(REPrimitiveNode::Resource(self.0, self.1))
    }
}

impl Into<RENodeByComplexity> for Bucket {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Primitive(REPrimitiveNode::Bucket(self))
    }
}

impl Into<RENodeByComplexity> for Proof {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Primitive(REPrimitiveNode::Proof(self))
    }
}

impl Into<RENodeByComplexity> for Vault {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Primitive(REPrimitiveNode::Vault(self))
    }
}

impl Into<RENodeByComplexity> for Worktop {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Primitive(REPrimitiveNode::Worktop(self))
    }
}

impl Into<RENodeByComplexity> for PreCommittedKeyValueStore {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Primitive(REPrimitiveNode::KeyValue(self))
    }
}

impl Into<RENodeByComplexity> for ValidatedPackage {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Primitive(REPrimitiveNode::Package(self))
    }
}

impl Into<RENodeByComplexity> for (Component, ComponentState) {
    fn into(self) -> RENodeByComplexity {
        RENodeByComplexity::Complex(REComplexValue::Component(self.0, self.1))
    }
}
