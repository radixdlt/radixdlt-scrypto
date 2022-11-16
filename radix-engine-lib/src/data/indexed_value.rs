use sbor::path::{SborPath, SborPathBuf};
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::component::{ComponentAddress, PackageAddress, SystemAddress};
use crate::core::Expression;
use crate::crypto::Blob;
use crate::data::*;
use crate::engine::types::*;
use crate::resource::{NonFungibleAddress, ResourceAddress};
use utils::misc::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ScryptoValueDecodeError {
    DecodeError(DecodeError),
    ValueIndexingError(ValueIndexingError),
}

pub enum ValueReplacingError {
    ProofIdNotFound(ProofId),
    BucketIdNotFound(BucketId),
}

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    pub raw: Vec<u8>,
    pub dom: ScryptoValue,

    // Global addresses
    pub component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub package_addresses: HashSet<PackageAddress>,
    pub system_addresses: HashSet<SystemAddress>,

    // RE nodes and refs
    pub bucket_ids: HashMap<BucketId, SborPath>,
    pub proof_ids: HashMap<ProofId, SborPath>,
    pub vault_ids: HashSet<VaultId>,
    pub kv_store_ids: HashSet<KeyValueStoreId>,
    pub component_ids: HashSet<ComponentId>,

    // Other interpreted
    pub expressions: Vec<(Expression, SborPath)>,
    pub blobs: Vec<(Blob, SborPath)>,
    pub non_fungible_addresses: HashSet<NonFungibleAddress>,
}

impl IndexedScryptoValue {
    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: Encode<ScryptoCustomTypeId>>(value: &T) -> Self {
        let bytes = encode(value);
        Self::from_slice(&bytes).expect("Failed to convert trusted value into IndexedScryptoValue")
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, ScryptoValueDecodeError> {
        let value = decode_any(slice).map_err(ScryptoValueDecodeError::DecodeError)?;
        Self::from_value(value)
    }

    pub fn from_value(value: ScryptoValue) -> Result<Self, ScryptoValueDecodeError> {
        let mut visitor = ScryptoCustomValueVisitor::new();
        let index_result = traverse_any(&mut SborPathBuf::new(), &value, &mut visitor);
        if let Err(error) = index_result {
            return Err(ScryptoValueDecodeError::ValueIndexingError(error));
        }

        Ok(Self {
            raw: encode_any(&value),
            dom: value,
            component_addresses: visitor.component_addresses,
            resource_addresses: visitor.resource_addresses,
            package_addresses: visitor.package_addresses,
            system_addresses: visitor.system_addresses,
            bucket_ids: visitor.buckets,
            proof_ids: visitor.proofs,
            vault_ids: visitor.vaults,
            kv_store_ids: visitor.kv_stores,
            component_ids: visitor.components,
            expressions: visitor.expressions,
            blobs: visitor.blobs,
            non_fungible_addresses: visitor.non_fungible_addresses,
        })
    }

    pub fn node_ids(&self) -> HashSet<RENodeId> {
        let mut node_ids = HashSet::new();
        for vault_id in &self.vault_ids {
            node_ids.insert(RENodeId::Vault(*vault_id));
        }
        for kv_store_id in &self.kv_store_ids {
            node_ids.insert(RENodeId::KeyValueStore(*kv_store_id));
        }
        for component_id in &self.component_ids {
            node_ids.insert(RENodeId::Component(*component_id));
        }
        for (bucket_id, _) in &self.bucket_ids {
            node_ids.insert(RENodeId::Bucket(*bucket_id));
        }
        for (proof_id, _) in &self.proof_ids {
            node_ids.insert(RENodeId::Proof(*proof_id));
        }
        node_ids
    }

    pub fn global_references(&self) -> HashSet<GlobalAddress> {
        let mut node_ids = HashSet::new();
        for component_address in &self.component_addresses {
            node_ids.insert(GlobalAddress::Component(*component_address));
        }
        for resource_address in &self.resource_addresses {
            node_ids.insert(GlobalAddress::Resource(*resource_address));
        }
        for non_fungible_address in &self.non_fungible_addresses {
            node_ids.insert(GlobalAddress::Resource(
                non_fungible_address.resource_address(),
            ));
        }
        for package_address in &self.package_addresses {
            node_ids.insert(GlobalAddress::Package(*package_address));
        }
        for system_address in &self.system_addresses {
            node_ids.insert(GlobalAddress::System(*system_address));
        }
        node_ids
    }

    pub fn replace_ids(
        &mut self,
        proof_replacements: &mut HashMap<ProofId, ProofId>,
        bucket_replacements: &mut HashMap<BucketId, BucketId>,
    ) -> Result<(), ValueReplacingError> {
        let mut new_proof_ids = HashMap::new();
        for (proof_id, path) in self.proof_ids.drain() {
            let next_id = proof_replacements
                .remove(&proof_id)
                .ok_or(ValueReplacingError::ProofIdNotFound(proof_id))?;
            let value = path.get_from_value_mut(&mut self.dom).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Proof(next_id);
            } else {
                panic!("Should be a custom value");
            }

            new_proof_ids.insert(next_id, path);
        }
        self.proof_ids = new_proof_ids;

        let mut new_bucket_ids = HashMap::new();
        for (bucket_id, path) in self.bucket_ids.drain() {
            let next_id = bucket_replacements
                .remove(&bucket_id)
                .ok_or(ValueReplacingError::BucketIdNotFound(bucket_id))?;
            let value = path.get_from_value_mut(&mut self.dom).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Bucket(next_id);
            } else {
                panic!("Should be a custom value");
            }

            new_bucket_ids.insert(next_id, path);
        }
        self.bucket_ids = new_bucket_ids;

        self.raw = encode_any(&self.dom);

        Ok(())
    }

    pub fn value_count(&self) -> usize {
        self.bucket_ids.len()
            + self.proof_ids.len()
            + self.vault_ids.len()
            + self.component_ids.len()
            + self.kv_store_ids.len()
    }
}

impl fmt::Debug for IndexedScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_scrypto_value(f, &self.dom, &ValueFormattingContext::no_context())
    }
}

impl<'a> ContextualDisplay<ValueFormattingContext<'a>> for IndexedScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ValueFormattingContext<'a>,
    ) -> Result<(), Self::Error> {
        format_scrypto_value(f, &self.dom, context)
    }
}

/// A visitor the indexes scrypto custom values.
pub struct ScryptoCustomValueVisitor {
    // Global addresses
    pub component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub package_addresses: HashSet<PackageAddress>,
    pub system_addresses: HashSet<SystemAddress>,
    // RE nodes
    pub buckets: HashMap<BucketId, SborPath>,
    pub proofs: HashMap<ProofId, SborPath>,
    pub vaults: HashSet<VaultId>,
    pub kv_stores: HashSet<KeyValueStoreId>,
    pub components: HashSet<ComponentId>,
    // Other interpreted
    pub expressions: Vec<(Expression, SborPath)>,
    pub blobs: Vec<(Blob, SborPath)>,
    pub non_fungible_addresses: HashSet<NonFungibleAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ValueIndexingError {
    DuplicateOwnership,
}

impl ScryptoCustomValueVisitor {
    pub fn new() -> Self {
        Self {
            component_addresses: HashSet::new(),
            resource_addresses: HashSet::new(),
            package_addresses: HashSet::new(),
            system_addresses: HashSet::new(),
            buckets: HashMap::new(),
            proofs: HashMap::new(),
            vaults: HashSet::new(),
            kv_stores: HashSet::new(),
            components: HashSet::new(),
            expressions: Vec::new(),
            blobs: Vec::new(),
            non_fungible_addresses: HashSet::new(),
        }
    }
}

impl CustomValueVisitor<ScryptoCustomTypeId, ScryptoCustomValue> for ScryptoCustomValueVisitor {
    type Err = ValueIndexingError;

    fn visit(
        &mut self,
        path: &mut SborPathBuf,
        value: &ScryptoCustomValue,
    ) -> Result<(), Self::Err> {
        match value {
            // Global addresses
            ScryptoCustomValue::PackageAddress(value) => {
                self.package_addresses.insert(value.clone());
            }
            ScryptoCustomValue::ComponentAddress(value) => {
                self.component_addresses.insert(value.clone());
            }
            ScryptoCustomValue::ResourceAddress(value) => {
                self.resource_addresses.insert(value.clone());
            }
            ScryptoCustomValue::SystemAddress(value) => {
                self.system_addresses.insert(value.clone());
            }

            // RE nodes & references
            ScryptoCustomValue::Component(value) => {
                if !self.components.insert(value.clone()) {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::KeyValueStore(value) => {
                if !self.kv_stores.insert(value.clone()) {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::Bucket(value) => {
                if self
                    .buckets
                    .insert(value.clone(), path.clone().into())
                    .is_some()
                {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::Proof(value) => {
                if self
                    .proofs
                    .insert(value.clone(), path.clone().into())
                    .is_some()
                {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::Vault(value) => {
                if !self.vaults.insert(value.clone()) {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }

            // Other interpreted
            ScryptoCustomValue::Expression(value) => {
                self.expressions.push((value.clone(), path.clone().into()));
            }
            ScryptoCustomValue::Blob(value) => {
                self.blobs.push((value.clone(), path.clone().into()));
            }
            ScryptoCustomValue::NonFungibleAddress(value) => {
                self.non_fungible_addresses.insert(value.clone());
            }

            // Uninterpreted
            ScryptoCustomValue::Hash(_)
            | ScryptoCustomValue::EcdsaSecp256k1PublicKey(_)
            | ScryptoCustomValue::EcdsaSecp256k1Signature(_)
            | ScryptoCustomValue::EddsaEd25519PublicKey(_)
            | ScryptoCustomValue::EddsaEd25519Signature(_)
            | ScryptoCustomValue::Decimal(_)
            | ScryptoCustomValue::PreciseDecimal(_)
            | ScryptoCustomValue::NonFungibleId(_) => {
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

    /// Encodes a data structure into byte array.
    fn scrypto_encode<T: Encode<ScryptoCustomTypeId> + ?Sized>(v: &T) -> Vec<u8> {
        encode(v)
    }

    #[test]
    fn should_reject_duplicate_ids() {
        let buckets = scrypto_encode(&vec![
            scrypto::resource::Bucket(0),
            scrypto::resource::Bucket(0),
        ]);
        assert_eq!(
            IndexedScryptoValue::from_slice(&buckets),
            Err(ScryptoValueDecodeError::ValueIndexingError(
                ValueIndexingError::DuplicateOwnership
            ))
        );
    }
}
