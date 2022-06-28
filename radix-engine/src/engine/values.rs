use sbor::rust::cell::{RefCell, RefMut};
use sbor::rust::collections::hash_map::IntoIter;
use sbor::rust::vec::Vec;
use scrypto::values::ScryptoValue;
use sbor::rust::collections::*;
use scrypto::engine::types::*;

use crate::engine::*;
use crate::model::*;

#[derive(Debug)]
pub enum REComplexValue {
    Component(Component)
}

impl REComplexValue {
    pub fn get_children(&self) -> Result<HashSet<ValueId>, RuntimeError> {
        match self {
            REComplexValue::Component(component) => {
                let value =
                    ScryptoValue::from_slice(component.state()).map_err(RuntimeError::DecodeError)?;
                Ok(value.value_ids())
            }
        }
    }

    pub fn into_re_value(self, children: HashMap<StoredValueId, StoredValue>) -> REValue {
        match self {
            REComplexValue::Component(component) => {
                REValue::Stored(StoredValue::Component {
                    component,
                    child_values: InMemoryChildren::with_values(children),
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum REPrimitiveValue {
    Package(ValidatedPackage),
    Bucket(Bucket),
    Proof(Proof),
    KeyValue(PreCommittedKeyValueStore),
    Vault(Vault),
}

#[derive(Debug)]
pub enum REValueByComplexity {
    Primitive(REPrimitiveValue),
    Complex(REComplexValue),
}

impl Into<REValue> for REPrimitiveValue {
    fn into(self) -> REValue {
        match self {
            REPrimitiveValue::Package(package) => REValue::Package(package),
            REPrimitiveValue::Bucket(bucket) => REValue::Transient(TransientValue::Bucket(bucket)),
            REPrimitiveValue::Proof(proof) => REValue::Transient(TransientValue::Proof(proof)),
            REPrimitiveValue::KeyValue(store) => {
                REValue::Stored(StoredValue::KeyValueStore {
                    store: store,
                    child_values: InMemoryChildren::new(),
                })
            },
            REPrimitiveValue::Vault(vault) => {
                REValue::Stored(StoredValue::Vault(vault))
            }
        }
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

impl Into<REValueByComplexity> for Component {
    fn into(self) -> REValueByComplexity {
        REValueByComplexity::Complex(REComplexValue::Component(self))
    }
}


#[derive(Debug)]
pub struct InMemoryChildren {
    child_values: HashMap<StoredValueId, RefCell<StoredValue>>,
}

impl InMemoryChildren {
    pub fn new() -> Self {
        InMemoryChildren {
            child_values: HashMap::new(),
        }
    }

    pub fn with_values(values: HashMap<StoredValueId, StoredValue>) -> Self {
        let mut child_values = HashMap::new();
        for (id, value) in values.into_iter() {
            child_values.insert(id, RefCell::new(value));
        }
        InMemoryChildren { child_values }
    }

    pub fn into_iter(self) -> IntoIter<StoredValueId, RefCell<StoredValue>> {
        self.child_values.into_iter()
    }

    pub fn all_descendants(&self) -> Vec<StoredValueId> {
        let mut descendents = Vec::new();
        for (id, value) in self.child_values.iter() {
            descendents.push(*id);
            let value = value.borrow();
            descendents.extend(value.all_descendants());
        }
        descendents
    }

    pub fn get_child(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> RefMut<StoredValue> {
        if ancestors.is_empty() {
            let value = self
                .child_values
                .get_mut(id)
                .expect("Value expected to exist");
            return value.borrow_mut();
        }

        let (first, rest) = ancestors.split_first().unwrap();
        let value = self
            .child_values
            .get_mut(&StoredValueId::KeyValueStoreId(*first))
            .unwrap();
        value.get_mut().get_child(rest, id)
    }

    pub fn get_child_mut(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> &mut StoredValue {
        if ancestors.is_empty() {
            let value = self
                .child_values
                .get_mut(id)
                .expect("Value expected to exist");
            return value.get_mut();
        }

        let (first, rest) = ancestors.split_first().unwrap();
        let value = self
            .child_values
            .get_mut(&StoredValueId::KeyValueStoreId(*first))
            .unwrap();
        value.get_mut().get_child_mut(rest, id)
    }

    pub fn insert_children(&mut self, values: HashMap<StoredValueId, StoredValue>) {
        for (id, value) in values {
            self.child_values.insert(id, RefCell::new(value));
        }
    }
}

#[derive(Debug)]
pub enum StoredValue {
    KeyValueStore {
        store: PreCommittedKeyValueStore,
        child_values: InMemoryChildren,
    },
    Component {
        component: Component,
        child_values: InMemoryChildren,
    },
    Vault(Vault),
}

impl StoredValue {
    pub fn component(&self) -> &Component {
        match self {
            StoredValue::Component { component, .. } => component,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store(&self) -> &PreCommittedKeyValueStore {
        match self {
            StoredValue::KeyValueStore { store, .. } => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn kv_store_mut(&mut self) -> &mut PreCommittedKeyValueStore {
        match self {
            StoredValue::KeyValueStore { store, .. } => store,
            _ => panic!("Expected to be a store"),
        }
    }

    pub fn vault(&self) -> &Vault {
        match self {
            StoredValue::Vault(vault) => vault,
            _ => panic!("Expected to be a vault"),
        }
    }

    pub fn all_descendants(&self) -> Vec<StoredValueId> {
        match self {
            StoredValue::KeyValueStore { child_values, .. }
            | StoredValue::Component { child_values, .. } => child_values.all_descendants(),
            StoredValue::Vault(..) => Vec::new(),
        }
    }

    pub fn get_child(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> RefMut<StoredValue> {
        match self {
            StoredValue::KeyValueStore { child_values, .. }
            | StoredValue::Component { child_values, .. } => child_values.get_child(ancestors, id),
            StoredValue::Vault(..) => panic!("Expected to be store"),
        }
    }

    pub fn get_child_mut(
        &mut self,
        ancestors: &[KeyValueStoreId],
        id: &StoredValueId,
    ) -> &mut StoredValue {
        match self {
            StoredValue::KeyValueStore { child_values, .. }
            | StoredValue::Component { child_values, .. } => {
                child_values.get_child_mut(ancestors, id)
            }
            StoredValue::Vault(..) => panic!("Expected to be store"),
        }
    }

    pub fn insert_children(&mut self, values: HashMap<StoredValueId, StoredValue>) {
        match self {
            StoredValue::KeyValueStore { child_values, .. }
            | StoredValue::Component { child_values, .. } => child_values.insert_children(values),
            StoredValue::Vault(..) => panic!("Expected to be store"),
        }
    }
}