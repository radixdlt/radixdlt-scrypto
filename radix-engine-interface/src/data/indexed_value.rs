use sbor::path::{SborPath, SborPathBuf};
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::api::types::*;
use crate::data::types::*;
use crate::data::*;
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ScryptoValueDecodeError {
    RawValueEncodeError(EncodeError),
    TypedValueEncodeError(EncodeError),
    DecodeError(DecodeError),
    ValueIndexingError(ValueIndexingError),
}

pub enum ReplaceManifestValuesError {
    ProofIdNotFound(ManifestProof),
    BucketIdNotFound(ManifestBucket),
}

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    pub value: ScryptoValue,

    // Global addresses
    pub component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub package_addresses: HashSet<PackageAddress>,
    pub system_addresses: HashSet<SystemAddress>,

    // RE interpreted
    pub owned_nodes: HashSet<Own>,
    pub blobs: Vec<(Blob, SborPath)>,

    // TX interpreted
    pub buckets: HashMap<ManifestBucket, SborPath>,
    pub proofs: HashMap<ManifestProof, SborPath>,
    pub expressions: Vec<(ManifestExpression, SborPath)>,
    pub bucket_arrays: Vec<SborPath>,
    pub proof_arrays: Vec<SborPath>,
}

impl IndexedScryptoValue {
    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let bytes =
            scrypto_encode(value).expect("Failed to encode trusted value for IndexedScryptoValue");
        Self::from_slice(&bytes).expect("Failed to convert trusted value into IndexedScryptoValue")
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, ScryptoValueDecodeError> {
        let value = scrypto_decode(slice).map_err(ScryptoValueDecodeError::DecodeError)?;
        Self::from_value(value)
    }

    pub fn from_value(value: ScryptoValue) -> Result<Self, ScryptoValueDecodeError> {
        let mut visitor = ScryptoValueVisitor::new();
        let index_result = traverse_any(&mut SborPathBuf::new(), &value, &mut visitor);
        if let Err(error) = index_result {
            return Err(ScryptoValueDecodeError::ValueIndexingError(error));
        }

        Ok(Self {
            value: value,
            component_addresses: visitor.component_addresses,
            resource_addresses: visitor.resource_addresses,
            package_addresses: visitor.package_addresses,
            system_addresses: visitor.system_addresses,

            owned_nodes: visitor.owned_nodes,
            blobs: visitor.blobs,

            buckets: visitor.buckets,
            proofs: visitor.proofs,
            expressions: visitor.expressions,
            bucket_arrays: visitor.bucket_arrays,
            proof_arrays: visitor.proof_arrays,
        })
    }

    pub fn as_vec(&self) -> Vec<u8> {
        scrypto_encode(&self.value).expect("Failed to encode IndexedScryptoValue")
    }

    pub fn as_typed<T: ScryptoDecode>(&self) -> Result<T, DecodeError> {
        let bytes = self.as_vec();
        scrypto_decode(&bytes)
    }

    pub fn owned_node_ids(&self) -> HashSet<RENodeId> {
        let mut node_ids = HashSet::new();
        for ownership in &self.owned_nodes {
            match ownership {
                Own::Bucket(bucket_id) => {
                    node_ids.insert(RENodeId::Bucket(*bucket_id));
                }
                Own::Proof(proof_id) => {
                    node_ids.insert(RENodeId::Proof(*proof_id));
                }
                Own::Vault(vault_id) => {
                    node_ids.insert(RENodeId::Vault(*vault_id));
                }
                Own::Component(component_id) => {
                    node_ids.insert(RENodeId::Component(*component_id));
                }
                Own::KeyValueStore(kv_store_id) => {
                    node_ids.insert(RENodeId::KeyValueStore(*kv_store_id));
                }
            }
        }
        node_ids
    }

    pub fn owned_node_count(&self) -> usize {
        self.owned_nodes.len()
    }

    pub fn global_references(&self) -> HashSet<GlobalAddress> {
        let mut node_ids = HashSet::new();
        for component_address in &self.component_addresses {
            node_ids.insert(GlobalAddress::Component(*component_address));
        }
        for resource_address in &self.resource_addresses {
            node_ids.insert(GlobalAddress::Resource(*resource_address));
        }
        for package_address in &self.package_addresses {
            node_ids.insert(GlobalAddress::Package(*package_address));
        }
        for system_address in &self.system_addresses {
            node_ids.insert(GlobalAddress::System(*system_address));
        }

        node_ids
    }

    pub fn replace_manifest_values(
        &mut self,
        proof_replacements: &mut HashMap<ManifestProof, ProofId>,
        bucket_replacements: &mut HashMap<ManifestBucket, BucketId>,
    ) -> Result<(), ReplaceManifestValuesError> {
        for (proof_id, path) in self.proofs.drain() {
            let next_id = proof_replacements
                .remove(&proof_id)
                .ok_or(ReplaceManifestValuesError::ProofIdNotFound(proof_id))?;
            let value = path.get_from_value_mut(&mut self.value).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Own(Own::Proof(next_id));
                self.owned_nodes.insert(Own::Proof(next_id));
            } else {
                panic!("Should be a custom value");
            }
        }

        for (bucket_id, path) in self.buckets.drain() {
            let next_id = bucket_replacements
                .remove(&bucket_id)
                .ok_or(ReplaceManifestValuesError::BucketIdNotFound(bucket_id))?;
            let value = path.get_from_value_mut(&mut self.value).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Own(Own::Bucket(next_id));
                self.owned_nodes.insert(Own::Bucket(next_id));
            } else {
                panic!("Should be a custom value");
            }
        }

        for path in self.bucket_arrays.drain(..) {
            let value = path.get_from_value_mut(&mut self.value).unwrap();
            if let SborValue::Array {
                element_type_id, ..
            } = value
            {
                *element_type_id = ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Own);
            } else {
                panic!("Should be an array");
            }
        }

        for path in self.proof_arrays.drain(..) {
            let value = path.get_from_value_mut(&mut self.value).unwrap();
            if let SborValue::Array {
                element_type_id, ..
            } = value
            {
                *element_type_id = ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Own);
            } else {
                panic!("Should be an array");
            }
        }

        Ok(())
    }
}

impl fmt::Debug for IndexedScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_scrypto_value(f, &self.value, &ValueFormattingContext::no_context())
    }
}

impl<'a> ContextualDisplay<ValueFormattingContext<'a>> for IndexedScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ValueFormattingContext<'a>,
    ) -> Result<(), Self::Error> {
        format_scrypto_value(f, &self.value, context)
    }
}

/// A visitor the indexes scrypto custom values.
pub struct ScryptoValueVisitor {
    // Global addresses
    pub component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub package_addresses: HashSet<PackageAddress>,
    pub system_addresses: HashSet<SystemAddress>,
    // RE interpreted
    pub owned_nodes: HashSet<Own>,
    pub blobs: Vec<(Blob, SborPath)>,
    // TX interpreted
    pub buckets: HashMap<ManifestBucket, SborPath>,
    pub proofs: HashMap<ManifestProof, SborPath>,
    pub expressions: Vec<(ManifestExpression, SborPath)>,
    pub bucket_arrays: Vec<SborPath>,
    pub proof_arrays: Vec<SborPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ValueIndexingError {
    DuplicateOwnership,
    DuplicateManifestBucket,
    DuplicateManifestProof,
}

impl ScryptoValueVisitor {
    pub fn new() -> Self {
        Self {
            component_addresses: HashSet::new(),
            resource_addresses: HashSet::new(),
            package_addresses: HashSet::new(),
            system_addresses: HashSet::new(),

            owned_nodes: HashSet::new(),
            blobs: Vec::new(),

            buckets: HashMap::new(),
            proofs: HashMap::new(),
            expressions: Vec::new(),
            bucket_arrays: Vec::new(),
            proof_arrays: Vec::new(),
        }
    }
}

impl ValueVisitor<ScryptoCustomTypeId, ScryptoCustomValue> for ScryptoValueVisitor {
    type Err = ValueIndexingError;

    fn visit_array(
        &mut self,
        path: &mut SborPathBuf,
        element_type_id: &ScryptoSborTypeId,
        _elements: &[ScryptoValue],
    ) -> Result<(), Self::Err> {
        match element_type_id {
            ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Bucket) => {
                self.bucket_arrays.push(path.clone().into());
            }
            ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Proof) => {
                self.proof_arrays.push(path.clone().into());
            }
            _ => {}
        }
        Ok(())
    }

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

            // RE interpreted
            ScryptoCustomValue::Own(value) => {
                if !self.owned_nodes.insert(value.clone()) {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::Blob(value) => {
                self.blobs.push((value.clone(), path.clone().into()));
            }

            // TX interpreted
            ScryptoCustomValue::Bucket(value) => {
                if self
                    .buckets
                    .insert(value.clone(), path.clone().into())
                    .is_some()
                {
                    return Err(ValueIndexingError::DuplicateManifestBucket);
                }
            }
            ScryptoCustomValue::Proof(value) => {
                if self
                    .proofs
                    .insert(value.clone(), path.clone().into())
                    .is_some()
                {
                    return Err(ValueIndexingError::DuplicateManifestProof);
                }
            }
            ScryptoCustomValue::Expression(value) => {
                self.expressions.push((value.clone(), path.clone().into()));
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

    #[test]
    fn should_reject_duplicate_ids() {
        let buckets = scrypto_encode(&vec![ManifestBucket(0), ManifestBucket(0)]).unwrap();
        assert_eq!(
            IndexedScryptoValue::from_slice(&buckets),
            Err(ScryptoValueDecodeError::ValueIndexingError(
                ValueIndexingError::DuplicateManifestBucket
            ))
        );
    }
}
