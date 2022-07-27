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
pub enum Address {
    GlobalComponent(ComponentAddress),
    Package(PackageAddress),
    ResourceManager(ResourceAddress),
    NonFungibleSpace(ResourceAddress),
    NonFungible(ResourceAddress, Vec<u8>),
    KeyValueStoreSpace(KeyValueStoreId),
    KeyValueStoreEntry(KeyValueStoreId, Vec<u8>),
    Vault(VaultId),
    LocalComponent(ComponentAddress),
    ComponentState(ComponentAddress),
    System,
}

impl Into<Address> for PackageAddress {
    fn into(self) -> Address {
        Address::Package(self)
    }
}

impl Into<Address> for ResourceAddress {
    fn into(self) -> Address {
        Address::ResourceManager(self)
    }
}

impl Into<Address> for VaultId {
    fn into(self) -> Address {
        Address::Vault(self)
    }
}

impl Into<PackageAddress> for Address {
    fn into(self) -> PackageAddress {
        if let Address::Package(package_address) = self {
            return package_address;
        } else {
            panic!("Address is not a package address");
        }
    }
}

impl Into<ComponentAddress> for Address {
    fn into(self) -> ComponentAddress {
        match self {
            Address::GlobalComponent(component_address)
            | Address::LocalComponent(component_address)
            | Address::ComponentState(component_address) => component_address,
            _ => panic!("Address is not a component address"),
        }
    }
}

impl Into<ResourceAddress> for Address {
    fn into(self) -> ResourceAddress {
        if let Address::ResourceManager(resource_address) = self {
            return resource_address;
        } else {
            panic!("Address is not a resource address");
        }
    }
}

impl Into<VaultId> for Address {
    fn into(self) -> VaultId {
        if let Address::Vault(id) = self {
            return id;
        } else {
            panic!("Address is not a vault address");
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
    Resource(ResourceManager),
    NonFungibles(HashMap<NonFungibleId, NonFungible>),
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
            RENode::Resource(resource_manager) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        match self {
            RENode::Resource(resource_manager) => resource_manager,
            _ => panic!("Expected to be a resource manager"),
        }
    }

    pub fn non_fungibles(&self) -> &HashMap<NonFungibleId, NonFungible> {
        match self {
            RENode::NonFungibles(non_fungibles) => non_fungibles,
            _ => panic!("Expected to be non fungibles"),
        }
    }

    pub fn non_fungibles_mut(&mut self) -> &mut HashMap<NonFungibleId, NonFungible> {
        match self {
            RENode::NonFungibles(non_fungibles) => non_fungibles,
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
            RENode::NonFungibles(..) => Ok(()),
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
            RENode::NonFungibles(..) => Err(RuntimeError::ValueNotAllowed),
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
            RENode::NonFungibles(..) => Err(DropFailure::Resource),
            RENode::System(..) => Err(DropFailure::System),
            RENode::Proof(proof) => {
                proof.drop();
                Ok(())
            }
            RENode::Worktop(worktop) => worktop.drop(),
        }
    }

    pub fn drop_values(values: Vec<REValue>) -> Result<(), DropFailure> {
        let mut worktops = Vec::new();
        for value in values {
            if let RENode::Worktop(worktop) = value.root {
                worktops.push(worktop);
            } else {
                value.try_drop()?;
            }
        }
        for worktop in worktops {
            worktop.drop()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct REValue {
    pub root: RENode,
    pub non_root_nodes: HashMap<ValueId, RENode>,
}

impl REValue {
    pub fn root(&self) -> &RENode {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut RENode {
        &mut self.root
    }

    pub fn non_root(&self, id: &ValueId) -> &RENode {
        self.non_root_nodes.get(id).unwrap()
    }

    pub fn non_root_mut(&mut self, id: &ValueId) -> &mut RENode {
        self.non_root_nodes.get_mut(id).unwrap()
    }

    pub fn get_node(&self, id: Option<&ValueId>) -> &RENode {
        if let Some(value_id) = id {
            self.non_root_nodes.get(value_id).unwrap()
        } else {
            &self.root
        }
    }

    pub fn get_node_mut(&mut self, id: Option<&ValueId>) -> &mut RENode {
        if let Some(value_id) = id {
            self.non_root_nodes.get_mut(value_id).unwrap()
        } else {
            &mut self.root
        }
    }

    pub fn insert_non_root_nodes(&mut self, values: HashMap<ValueId, RENode>) {
        for (id, value) in values {
            self.non_root_nodes.insert(id, value);
        }
    }

    pub fn to_nodes(self, root_id: ValueId) -> HashMap<ValueId, RENode> {
        let mut nodes = self.non_root_nodes;
        nodes.insert(root_id, self.root);
        nodes
    }

    pub fn try_drop(self) -> Result<(), DropFailure> {
        self.root.try_drop()
    }
}

impl Into<Bucket> for REValue {
    fn into(self) -> Bucket {
        match self.root {
            RENode::Bucket(bucket) => bucket,
            _ => panic!("Expected to be a bucket"),
        }
    }
}

impl Into<Proof> for REValue {
    fn into(self) -> Proof {
        match self.root {
            RENode::Proof(proof) => proof,
            _ => panic!("Expected to be a proof"),
        }
    }
}

impl Into<HashMap<NonFungibleId, NonFungible>> for REValue {
    fn into(self) -> HashMap<NonFungibleId, NonFungible> {
        match self.root {
            RENode::NonFungibles(non_fungibles) => non_fungibles,
            _ => panic!("Expected to be non fungibles"),
        }
    }
}

#[derive(Debug)]
pub enum REComplexValue {
    Component(Component, ComponentState),
}

impl REComplexValue {
    pub fn get_children(&self) -> Result<HashSet<ValueId>, RuntimeError> {
        match self {
            REComplexValue::Component(_, component_state) => {
                let value = ScryptoValue::from_slice(component_state.state())
                    .map_err(RuntimeError::DecodeError)?;
                Ok(value.value_ids())
            }
        }
    }

    pub fn into_re_value(self, non_root_values: HashMap<ValueId, REValue>) -> REValue {
        let mut non_root_nodes = HashMap::new();
        for (id, val) in non_root_values {
            non_root_nodes.extend(val.to_nodes(id));
        }
        match self {
            REComplexValue::Component(component, component_state) => REValue {
                root: RENode::Component(component, component_state),
                non_root_nodes,
            },
        }
    }
}

#[derive(Debug)]
pub enum REPrimitiveValue {
    Package(ValidatedPackage),
    Bucket(Bucket),
    Proof(Proof),
    KeyValue(PreCommittedKeyValueStore),
    Resource(ResourceManager),
    NonFungibles(ResourceAddress, HashMap<NonFungibleId, NonFungible>),
    Vault(Vault),
    Worktop(Worktop),
}

#[derive(Debug)]
pub enum REValueByComplexity {
    Primitive(REPrimitiveValue),
    Complex(REComplexValue),
}

impl Into<REValue> for REPrimitiveValue {
    fn into(self) -> REValue {
        let root = match self {
            REPrimitiveValue::Resource(resource_manager) => RENode::Resource(resource_manager),
            REPrimitiveValue::NonFungibles(_resource_address, non_fungibles) => {
                RENode::NonFungibles(non_fungibles)
            }
            REPrimitiveValue::Package(package) => RENode::Package(package),
            REPrimitiveValue::Bucket(bucket) => RENode::Bucket(bucket),
            REPrimitiveValue::Proof(proof) => RENode::Proof(proof),
            REPrimitiveValue::KeyValue(store) => RENode::KeyValueStore(store),
            REPrimitiveValue::Vault(vault) => RENode::Vault(vault),

            REPrimitiveValue::Worktop(worktop) => RENode::Worktop(worktop),
        };
        REValue {
            root,
            non_root_nodes: HashMap::new(),
        }
    }
}

impl Into<REValueByComplexity> for ResourceManager {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::Resource(self))
    }
}

impl Into<REValueByComplexity> for (ResourceAddress, HashMap<NonFungibleId, NonFungible>) {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::NonFungibles(self.0, self.1))
    }
}

impl Into<REValueByComplexity> for Bucket {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::Bucket(self))
    }
}

impl Into<REValueByComplexity> for Proof {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::Proof(self))
    }
}

impl Into<REValueByComplexity> for Vault {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::Vault(self))
    }
}

impl Into<REValueByComplexity> for Worktop {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::Worktop(self))
    }
}

impl Into<REValueByComplexity> for PreCommittedKeyValueStore {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::KeyValue(self))
    }
}

impl Into<REValueByComplexity> for ValidatedPackage {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Primitive(REPrimitiveValue::Package(self))
    }
}

impl Into<REValueByComplexity> for (Component, ComponentState) {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Complex(REComplexValue::Component(self.0, self.1))
    }
}
