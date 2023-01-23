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
use radix_engine_derive::*;
use utils::ContextualDisplay;

/// Represents an error when reading the owned node ids from a value.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ReadOwnedNodesError {
    DuplicateOwn,
}

/// Represents an error when replacing manifest values.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ReplaceManifestValuesError {
    BucketNotFound(ManifestBucket),
    ProofNotFound(ManifestProof),
    InvalidExpressionReplacements,
}

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    raw: Vec<u8>,
    value: ScryptoValue,

    // RE interpreted
    component_addresses: HashSet<ComponentAddress>,
    resource_addresses: HashSet<ResourceAddress>,
    package_addresses: HashSet<PackageAddress>,
    owned_nodes: Vec<(Own, SborPath)>,

    // TX interpreted
    buckets: Vec<(ManifestBucket, SborPath)>,
    proofs: Vec<(ManifestProof, SborPath)>,
    expressions: Vec<(ManifestExpression, SborPath)>,
    blobs: Vec<(ManifestBlobRef, SborPath)>,
    arrays: Vec<(ScryptoValueKind, SborPath)>,
    maps: Vec<(ScryptoValueKind, ScryptoValueKind, SborPath)>,
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
            component_addresses: visitor.component_addresses,
            resource_addresses: visitor.resource_addresses,
            package_addresses: visitor.package_addresses,

            owned_nodes: visitor.owned_nodes,
            blobs: visitor.blobs,

            buckets: visitor.buckets,
            proofs: visitor.proofs,
            expressions: visitor.expressions,
            arrays: visitor.arrays,
            maps: visitor.maps,
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

    pub fn buckets(&self) -> &Vec<(ManifestBucket, SborPath)> {
        &self.buckets
    }

    pub fn proofs(&self) -> &Vec<(ManifestProof, SborPath)> {
        &self.proofs
    }

    pub fn expressions(&self) -> &Vec<(ManifestExpression, SborPath)> {
        &self.expressions
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

        node_ids
    }

    // TODO: replace blobs with Vec<u8> so `Blob` can be used in argument list.
    pub fn replace_manifest_values(
        mut self,
        proof_replacements: &mut HashMap<ManifestProof, ProofId>,
        bucket_replacements: &mut HashMap<ManifestBucket, BucketId>,
        mut expression_replacements: Vec<Vec<Own>>,
    ) -> Result<Self, ReplaceManifestValuesError> {
        if expression_replacements.len() != self.expressions.len() {
            return Err(ReplaceManifestValuesError::InvalidExpressionReplacements);
        }

        // Array replacement:
        // * Vec<ManifestBucket> ==> Vec<Own>
        // * Vec<ManifestProof> ==> Vec<Own>
        // * Vec<ManifestExpression> ==> Vec<Vec<Own>>
        for (cached_element_value_kind, path) in &mut self.arrays {
            match cached_element_value_kind {
                ValueKind::Custom(ScryptoCustomValueKind::Bucket)
                | ValueKind::Custom(ScryptoCustomValueKind::Proof) => {
                    let value = path.get_from_value_mut(&mut self.value).unwrap();
                    if let Value::Array {
                        element_value_kind, ..
                    } = value
                    {
                        *element_value_kind = ScryptoValueKind::Custom(ScryptoCustomValueKind::Own);
                        *cached_element_value_kind =
                            ScryptoValueKind::Custom(ScryptoCustomValueKind::Own);
                    } else {
                        panic!("Should be an array");
                    }
                }
                ValueKind::Custom(ScryptoCustomValueKind::Expression) => {
                    let value = path.get_from_value_mut(&mut self.value).unwrap();
                    if let Value::Array {
                        element_value_kind, ..
                    } = value
                    {
                        *element_value_kind = ScryptoValueKind::Array;
                        *cached_element_value_kind = ScryptoValueKind::Array;
                    } else {
                        panic!("Should be an array");
                    }
                }
                _ => {}
            }
        }

        // Map replacement:
        // Map<K, ManifestBucket> => Map<K, Own>
        // Map<ManifestBucket, V> => Map<Own, V>
        // etc.
        for (cached_key_value_kind, cached_value_value_kind, path) in &mut self.maps {
            match cached_key_value_kind {
                ValueKind::Custom(ScryptoCustomValueKind::Bucket)
                | ValueKind::Custom(ScryptoCustomValueKind::Proof) => {
                    let value = path.get_from_value_mut(&mut self.value).unwrap();
                    if let Value::Map { key_value_kind, .. } = value {
                        *key_value_kind = ScryptoValueKind::Custom(ScryptoCustomValueKind::Own);
                        *cached_key_value_kind =
                            ScryptoValueKind::Custom(ScryptoCustomValueKind::Own);
                    } else {
                        panic!("Should be a map");
                    }
                }
                ValueKind::Custom(ScryptoCustomValueKind::Expression) => {
                    let value = path.get_from_value_mut(&mut self.value).unwrap();
                    if let Value::Map { key_value_kind, .. } = value {
                        *key_value_kind = ScryptoValueKind::Array;
                        *cached_key_value_kind = ScryptoValueKind::Array;
                    } else {
                        panic!("Should be a map");
                    }
                }
                _ => {}
            }
            match cached_value_value_kind {
                ValueKind::Custom(ScryptoCustomValueKind::Bucket)
                | ValueKind::Custom(ScryptoCustomValueKind::Proof) => {
                    let value = path.get_from_value_mut(&mut self.value).unwrap();
                    if let Value::Map {
                        value_value_kind, ..
                    } = value
                    {
                        *value_value_kind = ScryptoValueKind::Custom(ScryptoCustomValueKind::Own);
                        *cached_value_value_kind =
                            ScryptoValueKind::Custom(ScryptoCustomValueKind::Own);
                    } else {
                        panic!("Should be a map");
                    }
                }
                ValueKind::Custom(ScryptoCustomValueKind::Expression) => {
                    let value = path.get_from_value_mut(&mut self.value).unwrap();
                    if let Value::Map {
                        value_value_kind, ..
                    } = value
                    {
                        *value_value_kind = ScryptoValueKind::Array;
                        *cached_value_value_kind = ScryptoValueKind::Array;
                    } else {
                        panic!("Should be a map");
                    }
                }
                _ => {}
            }
        }

        // Bucket replacement
        for (bucket, path) in self.buckets.drain(..) {
            let replacement = bucket_replacements
                .remove(&bucket)
                .ok_or(ReplaceManifestValuesError::BucketNotFound(bucket))?;

            let value = path.get_from_value_mut(&mut self.value).unwrap();
            *value = ScryptoValue::Custom {
                value: ScryptoCustomValue::Own(Own::Bucket(replacement)),
            };

            // new own
            self.owned_nodes.push((Own::Bucket(replacement), path));
        }

        // Proof replacement
        for (proof, path) in self.proofs.drain(..) {
            let replacement = proof_replacements
                .remove(&proof)
                .ok_or(ReplaceManifestValuesError::ProofNotFound(proof))?;
            let value = path.get_from_value_mut(&mut self.value).unwrap();
            *value = ScryptoValue::Custom {
                value: ScryptoCustomValue::Own(Own::Proof(replacement)),
            };

            // new own
            self.owned_nodes.push((Own::Proof(replacement), path));
        }

        // Expression replacement
        for (_, path) in self.expressions.drain(..).rev() {
            let replacement = expression_replacements.pop().unwrap();

            let value = path.get_from_value_mut(&mut self.value).unwrap();
            let element_value_kind = ScryptoValueKind::Custom(ScryptoCustomValueKind::Own);
            let elements = replacement
                .iter()
                .map(|r| ScryptoValue::Custom {
                    value: ScryptoCustomValue::Own(r.clone()),
                })
                .collect();
            *value = ScryptoValue::Array {
                element_value_kind,
                elements,
            };

            // new array
            self.arrays.push((element_value_kind, path.clone()));

            // new owns
            replacement.into_iter().enumerate().for_each(|(i, o)| {
                let mut buf = SborPathBuf::from(path.clone());
                buf.push(i);
                self.owned_nodes.push((o, buf.into()));
            })
        }

        // Potential optimization: in-place replacement
        self.raw =
            scrypto_encode(&self.value).expect("Value no longer encodable after replacement");

        Ok(self)
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
    // RE interpreted
    pub component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub package_addresses: HashSet<PackageAddress>,
    pub owned_nodes: Vec<(Own, SborPath)>,
    // TX interpreted
    pub buckets: Vec<(ManifestBucket, SborPath)>,
    pub proofs: Vec<(ManifestProof, SborPath)>,
    pub expressions: Vec<(ManifestExpression, SborPath)>,
    pub blobs: Vec<(ManifestBlobRef, SborPath)>,
    pub arrays: Vec<(ScryptoValueKind, SborPath)>,
    pub maps: Vec<(ScryptoValueKind, ScryptoValueKind, SborPath)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
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

            owned_nodes: Vec::new(),
            blobs: Vec::new(),

            buckets: Vec::new(),
            proofs: Vec::new(),
            expressions: Vec::new(),
            arrays: Vec::new(),
            maps: Vec::new(),
        }
    }
}

impl ValueVisitor<ScryptoCustomValueKind, ScryptoCustomValue> for ScryptoValueVisitor {
    type Err = Infallible;

    fn visit_array(
        &mut self,
        path: &mut SborPathBuf,
        element_value_kind: &ScryptoValueKind,
        _elements: &[ScryptoValue],
    ) -> Result<(), Self::Err> {
        self.arrays
            .push((element_value_kind.clone(), path.clone().into()));
        Ok(())
    }

    fn visit_map(
        &mut self,
        path: &mut SborPathBuf,
        key_value_kind: &ScryptoValueKind,
        value_value_kind: &ScryptoValueKind,
        _entries: &[(ScryptoValue, ScryptoValue)],
    ) -> Result<(), Self::Err> {
        self.maps.push((
            key_value_kind.clone(),
            value_value_kind.clone(),
            path.clone().into(),
        ));
        Ok(())
    }

    fn visit(
        &mut self,
        path: &mut SborPathBuf,
        value: &ScryptoCustomValue,
    ) -> Result<(), Self::Err> {
        match value {
            // RE interpreted
            ScryptoCustomValue::PackageAddress(value) => {
                self.package_addresses.insert(value.clone());
            }
            ScryptoCustomValue::ComponentAddress(value) => {
                self.component_addresses.insert(value.clone());
            }
            ScryptoCustomValue::ResourceAddress(value) => {
                self.resource_addresses.insert(value.clone());
            }
            ScryptoCustomValue::Own(value) => {
                self.owned_nodes.push((value.clone(), path.clone().into()));
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
            ScryptoCustomValue::Blob(value) => {
                self.blobs.push((value.clone(), path.clone().into()));
            }

            // Uninterpreted
            ScryptoCustomValue::Hash(_)
            | ScryptoCustomValue::EcdsaSecp256k1PublicKey(_)
            | ScryptoCustomValue::EcdsaSecp256k1Signature(_)
            | ScryptoCustomValue::EddsaEd25519PublicKey(_)
            | ScryptoCustomValue::EddsaEd25519Signature(_)
            | ScryptoCustomValue::Decimal(_)
            | ScryptoCustomValue::PreciseDecimal(_)
            | ScryptoCustomValue::NonFungibleLocalId(_) => {
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
        let value = IndexedScryptoValue::from_typed(&vec![ManifestBucket(0), ManifestBucket(0)]);
        assert_eq!(
            value.replace_manifest_values(
                &mut HashMap::from([(ManifestProof(0), 0u32)]),
                &mut HashMap::from([(ManifestBucket(0), 0u32)]),
                Vec::new(),
            ),
            Err(ReplaceManifestValuesError::BucketNotFound(ManifestBucket(
                0
            )))
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
