use radix_common::data::scrypto::*;
use radix_common::types::*;
use radix_rust::ContextualDisplay;
use sbor::representations::*;
use sbor::rust::fmt;
use sbor::rust::prelude::*;
use sbor::traversal::*;
use sbor::*;

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue<'v> {
    value: ScryptoRawValue<'v>,
    references: Vec<NodeId>,
    owned_nodes: Vec<NodeId>,
}

pub type IndexedOwnedScryptoValue = IndexedScryptoValue<'static>;

impl<'v> IndexedScryptoValue<'v> {
    fn new_from_unvalidated(
        scrypto_value: ScryptoUnvalidatedRawValue<'v>,
    ) -> Result<Self, DecodeError> {
        let (references, owned_nodes) = Self::validate_and_extract_references_and_owned_nodes(
            scrypto_value.traverser(0, SCRYPTO_SBOR_V1_MAX_DEPTH),
        )?;
        let scrypto_value = scrypto_value.confirm_validated();
        Ok(Self {
            value: scrypto_value,
            references,
            owned_nodes,
        })
    }

    fn new(scrypto_value: ScryptoRawValue<'v>) -> Result<Self, DecodeError> {
        let (references, owned_nodes) = Self::validate_and_extract_references_and_owned_nodes(
            scrypto_value.traverser(0, SCRYPTO_SBOR_V1_MAX_DEPTH),
        )?;
        Ok(Self {
            value: scrypto_value,
            references,
            owned_nodes,
        })
    }

    fn validate_and_extract_references_and_owned_nodes(
        mut traverser: VecTraverser<ScryptoCustomTraversal>,
    ) -> Result<(Vec<NodeId>, Vec<NodeId>), DecodeError> {
        let mut references = Vec::<NodeId>::new();
        let mut owned_nodes = Vec::<NodeId>::new();
        loop {
            let event = traverser.next_event();
            match event.event {
                TraversalEvent::ContainerStart(_) => {}
                TraversalEvent::ContainerEnd(_) => {}
                TraversalEvent::TerminalValue(r) => {
                    if let traversal::TerminalValueRef::Custom(c) = r {
                        match c.0 {
                            ScryptoCustomValue::Reference(node_id) => {
                                references.push(node_id.0.into());
                            }
                            ScryptoCustomValue::Own(node_id) => {
                                owned_nodes.push(node_id.0.into());
                            }
                            ScryptoCustomValue::Decimal(_)
                            | ScryptoCustomValue::PreciseDecimal(_)
                            | ScryptoCustomValue::NonFungibleLocalId(_) => {}
                        }
                    }
                }
                TraversalEvent::TerminalValueBatch(_) => {}
                TraversalEvent::End => {
                    break;
                }
                TraversalEvent::DecodeError(e) => {
                    return Err(e);
                }
            }
        }
        Ok((references, owned_nodes))
    }

    pub fn ref_into_owned(&self) -> IndexedOwnedScryptoValue {
        IndexedOwnedScryptoValue {
            value: self.value.ref_into_owned(),
            references: self.references.clone(),
            owned_nodes: self.owned_nodes.clone(),
        }
    }

    pub fn into_owned(self) -> IndexedOwnedScryptoValue {
        IndexedOwnedScryptoValue {
            value: self.value.into_owned(),
            references: self.references,
            owned_nodes: self.owned_nodes,
        }
    }

    pub fn value(&self) -> &ScryptoRawValue<'v> {
        &self.value
    }

    pub fn as_value(&self) -> ScryptoRawValue {
        self.value.as_value_ref()
    }

    pub fn as_unvalidated(&self) -> ScryptoUnvalidatedRawValue {
        self.value.as_unvalidated()
    }

    pub fn as_payload(&self) -> ScryptoRawPayload {
        self.value.as_payload()
    }

    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let value = scrypto_encode_to_value(value).expect("Failed to encode trusted Rust value");
        Self::new(value).expect("Failed to index trusted Rust value")
    }

    pub fn from_value(value: ScryptoRawValue<'v>) -> Self {
        Self::new(value).expect("Failed to index trusted ScryptoRawValue")
    }

    pub fn from_payload(payload: ScryptoRawPayload<'v>) -> Self {
        Self::new(payload.into_value()).expect("Failed to index trusted ScryptoRawPayload")
    }

    pub fn from_untrusted_payload_slice(slice: &'v [u8]) -> Result<Self, DecodeError> {
        Self::new_from_unvalidated(ScryptoUnvalidatedRawValue::from_payload_slice(slice))
    }

    pub fn from_untrusted_payload_vec(vec: Vec<u8>) -> Result<Self, DecodeError> {
        Self::new_from_unvalidated(ScryptoUnvalidatedOwnedRawValue::from_payload(vec))
    }

    pub fn into_typed<T: ScryptoDecode>(&self) -> Result<T, DecodeError> {
        self.value().decode_as()
    }

    pub fn payload_len(&self) -> usize {
        self.value.payload_len()
    }

    pub fn references(&self) -> &Vec<NodeId> {
        &self.references
    }

    pub fn owned_nodes(&self) -> &Vec<NodeId> {
        &self.owned_nodes
    }

    pub fn into_value(self) -> ScryptoRawValue<'v> {
        self.value
    }

    pub fn into_payload(self) -> ScryptoRawPayload<'v> {
        self.value.into_payload()
    }

    pub fn into_payload_bytes(self) -> Vec<u8> {
        self.into_payload().into_bytes()
    }

    pub fn unpack(self) -> (ScryptoRawValue<'v>, Vec<NodeId>, Vec<NodeId>) {
        (self.value, self.owned_nodes, self.references)
    }
}

impl<'a> fmt::Debug for IndexedScryptoValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.display(ValueDisplayParameters::Schemaless {
                display_mode: DisplayMode::RustLike,
                print_mode: PrintMode::SingleLine,
                custom_context: ScryptoValueDisplayContext::no_context(),
                depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH
            })
        )
    }
}

impl<'s, 'a, 'b> ContextualDisplay<ValueDisplayParameters<'s, 'a, ScryptoCustomExtension>>
    for IndexedScryptoValue<'b>
{
    type Error = sbor::representations::FormattingError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ValueDisplayParameters<'_, '_, ScryptoCustomExtension>,
    ) -> Result<(), Self::Error> {
        self.value().format(f, *context)
    }
}
