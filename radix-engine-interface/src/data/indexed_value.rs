use sbor::path::{SborPath, SborPathBuf};
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::convert::Infallible;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::api::types::*;
use crate::data::types::*;
use crate::data::*;
use utils::ContextualDisplay;

/// Represents an error when reading the owned node ids from a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOwnedNodesError {
    DuplicateOwn,
}

/// Represents an error when replacing manifest values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplaceManifestValuesError {
    BucketNotExistsOrConsumed(ManifestBucket),
    ProofNotExistsOrConsumed(ManifestProof),
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
    pub owned_nodes: Vec<(Own, SborPath)>,
    pub blobs: Vec<(Blob, SborPath)>,

    // TX interpreted
    pub buckets: Vec<(ManifestBucket, SborPath)>,
    pub proofs: Vec<(ManifestProof, SborPath)>,
    pub expressions: Vec<(ManifestExpression, SborPath)>,
    pub bucket_arrays: Vec<SborPath>,
    pub proof_arrays: Vec<SborPath>,
}

impl IndexedScryptoValue {
    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let bytes = scrypto_encode(value).expect("Failed to encode trusted value");
        let value = scrypto_decode(&bytes).expect("Failed to decode trusted value");
        Self::from_value(value)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        let value = scrypto_decode(slice)?;
        Ok(Self::from_value(value))
    }

    pub fn from_value(value: ScryptoValue) -> Self {
        let mut visitor = ScryptoValueVisitor::new();
        traverse_any(&mut SborPathBuf::new(), &value, &mut visitor).expect("Infallible");

        Self {
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
        }
    }

    pub fn as_vec(&self) -> Vec<u8> {
        scrypto_encode(&self.value).expect("Failed to encode IndexedScryptoValue")
    }

    pub fn as_typed<T: ScryptoDecode>(&self) -> Result<T, DecodeError> {
        let bytes = self.as_vec();
        scrypto_decode(&bytes)
    }

    pub fn owned_node_ids(&self) -> Result<HashSet<RENodeId>, ReadOwnedNodesError> {
        let mut node_ids = HashSet::new();
        for (owned_node, _) in &self.owned_nodes {
            let newly_inserted = match owned_node {
                Own::Bucket(bucket_id) => node_ids.insert(RENodeId::Bucket(*bucket_id)),
                Own::Proof(proof_id) => node_ids.insert(RENodeId::Proof(*proof_id)),
                Own::Vault(vault_id) => node_ids.insert(RENodeId::Vault(*vault_id)),
                Own::Component(component_id) => node_ids.insert(RENodeId::Component(*component_id)),
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
        for (bucket_id, path) in self.buckets.drain(..) {
            let next_id = bucket_replacements.remove(&bucket_id).ok_or(
                ReplaceManifestValuesError::BucketNotExistsOrConsumed(bucket_id),
            )?;
            let value = path.get_from_value_mut(&mut self.value).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Own(Own::Bucket(next_id));
                self.owned_nodes.push((Own::Bucket(next_id), path));
            } else {
                panic!("Should be a custom value");
            }
        }

        for (proof_id, path) in self.proofs.drain(..) {
            let next_id = proof_replacements.remove(&proof_id).ok_or(
                ReplaceManifestValuesError::ProofNotExistsOrConsumed(proof_id),
            )?;
            let value = path.get_from_value_mut(&mut self.value).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Own(Own::Proof(next_id));
                self.owned_nodes.push((Own::Proof(next_id), path));
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
    pub owned_nodes: Vec<(Own, SborPath)>,
    pub blobs: Vec<(Blob, SborPath)>,
    // TX interpreted
    pub buckets: Vec<(ManifestBucket, SborPath)>,
    pub proofs: Vec<(ManifestProof, SborPath)>,
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

            owned_nodes: Vec::new(),
            blobs: Vec::new(),

            buckets: Vec::new(),
            proofs: Vec::new(),
            expressions: Vec::new(),
            bucket_arrays: Vec::new(),
            proof_arrays: Vec::new(),
        }
    }
}

impl ValueVisitor<ScryptoCustomTypeId, ScryptoCustomValue> for ScryptoValueVisitor {
    type Err = Infallible;

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
                self.owned_nodes.push((value.clone(), path.clone().into()));
            }
            ScryptoCustomValue::Blob(value) => {
                self.blobs.push((value.clone(), path.clone().into()));
            }

            // TX interpreted
            ScryptoCustomValue::Bucket(value) => {
                self.buckets.push((value.clone(), path.clone().into()));
            }
            ScryptoCustomValue::Proof(value) => {
                self.proofs.push((value.clone(), path.clone().into()));
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
    fn should_reject_duplicate_manifest_buckets() {
        let mut value =
            IndexedScryptoValue::from_typed(&vec![ManifestBucket(0), ManifestBucket(0)]);
        assert_eq!(
            value.replace_manifest_values(
                &mut HashMap::from([(ManifestProof(0), 0u32)]),
                &mut HashMap::from([(ManifestBucket(0), 0u32)])
            ),
            Err(ReplaceManifestValuesError::BucketNotExistsOrConsumed(
                ManifestBucket(0)
            ))
        );
    }

    #[test]
    fn should_reject_duplicate_owned_buckets() {
        let value = IndexedScryptoValue::from_typed(&vec![Bucket(0), Bucket(0)]);
        assert_eq!(
            value.owned_node_ids(),
            Err(ReadOwnedNodesError::DuplicateOwn)
        );
    }
}
