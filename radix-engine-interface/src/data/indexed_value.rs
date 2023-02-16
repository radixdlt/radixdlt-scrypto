use crate::api::types::*;
use crate::data::model::*;
use crate::data::*;
use radix_engine_derive::*;
use sbor::path::{SborPath, SborPathBuf};
use sbor::rust::collections::HashSet;
use sbor::rust::convert::Infallible;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use utils::ContextualDisplay;

/// Represents an error when reading the owned node ids from a value.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ReadOwnedNodesError {
    DuplicateOwn,
}

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    raw: Vec<u8>,
    value: ScryptoValue,

    references: Vec<(Address, SborPath)>,
    owned_nodes: Vec<(Own, SborPath)>,
}

impl Into<ScryptoValue> for IndexedScryptoValue {
    fn into(self) -> ScryptoValue {
        self.value
    }
}

impl IndexedScryptoValue {
    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let bytes = scrypto_encode(value).expect("Failed to encode rust value");
        let value = scrypto_decode(&bytes).expect("Failed to decode rust value");
        Self::new(bytes, value)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        let value = scrypto_decode(slice)?;
        Ok(Self::new(slice.to_vec(), value))
    }

    pub fn from_vec(vec: Vec<u8>) -> Result<Self, DecodeError> {
        let value = scrypto_decode(&vec)?;
        Ok(Self::new(vec, value))
    }

    pub fn from_value(value: ScryptoValue) -> Self {
        let bytes = scrypto_encode(&value).expect("Failed to decode scrypto value");
        Self::new(bytes, value)
    }

    fn new(raw: Vec<u8>, value: ScryptoValue) -> Self {
        let mut visitor = ScryptoValueVisitor::new();
        traverse_any(&mut SborPathBuf::new(), &value, &mut visitor).expect("Infallible");

        Self {
            raw: raw,
            value: value,
            references: visitor.references,
            owned_nodes: visitor.owned_nodes,
        }
    }

    pub fn as_typed<T: ScryptoDecode>(&self) -> Result<T, DecodeError> {
        scrypto_decode(&self.raw)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.raw.as_slice()
    }

    pub fn as_value(&self) -> &ScryptoValue {
        &self.value
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.raw.clone()
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.raw
    }

    pub fn owned_node_ids(&self) -> Result<HashSet<RENodeId>, ReadOwnedNodesError> {
        let mut node_ids = HashSet::new();
        for (owned_node, _) in &self.owned_nodes {
            let newly_inserted = match owned_node {
                Own::Bucket(bucket_id) => node_ids.insert(RENodeId::Bucket(*bucket_id)),
                Own::Proof(proof_id) => node_ids.insert(RENodeId::Proof(*proof_id)),
                Own::Vault(vault_id) => node_ids.insert(RENodeId::Vault(*vault_id)),
                Own::Component(component_id) => node_ids.insert(RENodeId::Component(*component_id)),
                Own::Account(component_id) => node_ids.insert(RENodeId::Account(*component_id)),
                Own::KeyValueStore(kv_store_id) => {
                    node_ids.insert(RENodeId::KeyValueStore(*kv_store_id))
                }
            };
            if !newly_inserted {
                return Err(ReadOwnedNodesError::DuplicateOwn);
            }
        }
        Ok(node_ids)
    }

    pub fn global_references(&self) -> HashSet<GlobalAddress> {
        let mut references = HashSet::new();
        for (reference, _) in &self.references {
            match reference {
                Address::Package(address) => {
                    references.insert(GlobalAddress::Package(*address));
                }
                Address::Component(address) => {
                    references.insert(GlobalAddress::Component(*address));
                }
                Address::ResourceManager(address) => {
                    references.insert(GlobalAddress::Resource(*address));
                }
            }
        }

        references
    }
}

impl fmt::Debug for IndexedScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_scrypto_value(f, &self.value, &ScryptoValueDisplayContext::no_context())
    }
}

impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for IndexedScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_scrypto_value(f, &self.value, context)
    }
}

/// A visitor the indexes scrypto custom values.
pub struct ScryptoValueVisitor {
    pub references: Vec<(Address, SborPath)>,
    pub owned_nodes: Vec<(Own, SborPath)>,
}

impl ScryptoValueVisitor {
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
            owned_nodes: Vec::new(),
        }
    }
}

impl ValueVisitor<ScryptoCustomValueKind, ScryptoCustomValue> for ScryptoValueVisitor {
    type Err = Infallible;

    fn visit(
        &mut self,
        path: &mut SborPathBuf,
        value: &ScryptoCustomValue,
    ) -> Result<(), Self::Err> {
        match value {
            ScryptoCustomValue::Address(value) => {
                self.references.push((value.clone(), path.clone().into()));
            }
            ScryptoCustomValue::Own(value) => {
                self.owned_nodes.push((value.clone(), path.clone().into()));
            }

            ScryptoCustomValue::Decimal(_)
            | ScryptoCustomValue::PreciseDecimal(_)
            | ScryptoCustomValue::NonFungibleLocalId(_)
            | ScryptoCustomValue::PublicKey(_) => {
                // no-op
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::rust::vec;
    use super::*;

    #[test]
    fn should_reject_duplicate_owned_buckets() {
        let value =
            IndexedScryptoValue::from_typed(&vec![Own::Bucket([0u8; 36]), Own::Bucket([0u8; 36])]);
        assert_eq!(
            value.owned_node_ids(),
            Err(ReadOwnedNodesError::DuplicateOwn)
        );
    }
}
