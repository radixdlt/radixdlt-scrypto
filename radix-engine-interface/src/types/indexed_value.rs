use core::cell::RefCell;
use radix_common::data::scrypto::*;
use radix_common::types::*;
use radix_rust::ContextualDisplay;
use sbor::representations::*;
use sbor::rust::cell::Ref;
use sbor::rust::fmt;
use sbor::rust::prelude::*;
use sbor::traversal::*;
use sbor::*;

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    bytes: Vec<u8>,
    references: Vec<NodeId>,
    owned_nodes: Vec<NodeId>,
    scrypto_value: RefCell<Option<ScryptoValue>>,
}

#[derive(Default)]
struct OwnedAndReferenceAggregator {
    references: Vec<NodeId>,
    owned_nodes: Vec<NodeId>,
}

impl<'de> UntypedPayloadVisitor<'de, ScryptoCustomTraversal> for OwnedAndReferenceAggregator {
    type Output<'t> = Result<(), DecodeError>;

    fn on_terminal_value<'t>(
        &mut self,
        details: OnTerminalValue<'t, 'de, ScryptoCustomTraversal>,
    ) -> core::ops::ControlFlow<Self::Output<'t>> {
        if let traversal::TerminalValueRef::Custom(custom) = details.value {
            match custom.0 {
                ScryptoCustomValue::Reference(node_id) => {
                    self.references.push(node_id.0.into());
                }
                ScryptoCustomValue::Own(node_id) => {
                    self.owned_nodes.push(node_id.0.into());
                }
                ScryptoCustomValue::Decimal(_)
                | ScryptoCustomValue::PreciseDecimal(_)
                | ScryptoCustomValue::NonFungibleLocalId(_) => {}
            }
        }
        core::ops::ControlFlow::Continue(())
    }

    fn on_error<'t>(&mut self, details: OnError<'t, ScryptoCustomTraversal>) -> Self::Output<'t> {
        Err(details.error)
    }

    fn on_traversal_end<'t>(
        &mut self,
        _details: OnTraversalEnd<'t, ScryptoCustomTraversal>,
    ) -> Self::Output<'t> {
        Ok(())
    }
}

impl IndexedScryptoValue {
    fn new(bytes: Vec<u8>) -> Result<Self, DecodeError> {
        let mut aggregates = OwnedAndReferenceAggregator::default();
        ScryptoUntypedTraverser::new(
            &bytes,
            UntypedTraverserConfig {
                max_depth: SCRYPTO_SBOR_V1_MAX_DEPTH,
                check_exact_end: true,
            },
        )
        .run_from_start(
            ExpectedStart::PayloadPrefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX),
            &mut aggregates,
        )?;

        Ok(Self {
            bytes,
            references: aggregates.references,
            owned_nodes: aggregates.owned_nodes,
            scrypto_value: RefCell::new(None),
        })
    }

    fn get_scrypto_value(&self) -> Ref<ScryptoValue> {
        let is_empty = { self.scrypto_value.borrow().is_none() };

        if is_empty {
            *self.scrypto_value.borrow_mut() = Some(
                scrypto_decode::<ScryptoValue>(&self.bytes)
                    .expect("Failed to decode bytes in IndexedScryptoValue"),
            );
        }

        Ref::map(self.scrypto_value.borrow(), |v| v.as_ref().unwrap())
    }

    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let bytes = scrypto_encode(value).expect("Failed to encode trusted Rust value");
        Self::new(bytes).expect("Failed to index trusted Rust value")
    }

    pub fn from_scrypto_value(value: ScryptoValue) -> Self {
        let bytes = scrypto_encode(&value).expect("Failed to encode trusted ScryptoValue");
        Self::new(bytes).expect("Failed to index trusted ScryptoValue")
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        Self::new(slice.to_vec())
    }

    pub fn from_vec(vec: Vec<u8>) -> Result<Self, DecodeError> {
        Self::new(vec)
    }

    pub fn to_scrypto_value(&self) -> ScryptoValue {
        self.get_scrypto_value().clone()
    }

    pub fn as_scrypto_value(&self) -> Ref<ScryptoValue> {
        self.get_scrypto_value()
    }

    pub fn as_typed<T: ScryptoDecode>(&self) -> Result<T, DecodeError> {
        scrypto_decode(&self.bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn as_vec_ref(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn references(&self) -> &Vec<NodeId> {
        &self.references
    }

    pub fn owned_nodes(&self) -> &Vec<NodeId> {
        &self.owned_nodes
    }

    pub fn unpack(self) -> (Vec<u8>, Vec<NodeId>, Vec<NodeId>) {
        (self.bytes, self.owned_nodes, self.references)
    }
}

impl Into<Vec<u8>> for IndexedScryptoValue {
    fn into(self) -> Vec<u8> {
        self.bytes
    }
}

impl fmt::Debug for IndexedScryptoValue {
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

impl<'s, 'a> ContextualDisplay<ValueDisplayParameters<'s, 'a, ScryptoCustomExtension>>
    for IndexedScryptoValue
{
    type Error = sbor::representations::FormattingError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ValueDisplayParameters<'_, '_, ScryptoCustomExtension>,
    ) -> Result<(), Self::Error> {
        ScryptoRawPayload::new_from_valid_slice(self.as_slice()).format(f, *context)
    }
}
