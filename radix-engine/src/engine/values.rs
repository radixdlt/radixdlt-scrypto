use sbor::rust::collections::*;
use scrypto::engine::types::*;
use scrypto::values::*;

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