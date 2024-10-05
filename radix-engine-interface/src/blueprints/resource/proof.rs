use crate::blueprints::resource::*;
use crate::internal_prelude::*;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::ScryptoCustomTypeKind;
use radix_common::data::scrypto::ScryptoCustomValueKind;
use radix_common::data::scrypto::*;
use radix_common::types::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const PROOF_DROP_IDENT: &str = "Proof_drop";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProofDropInput {
    pub proof: Proof,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ProofDropManifestInput {
    pub proof: InternalAddress,
}

pub type ProofDropOutput = ();

pub const PROOF_GET_AMOUNT_IDENT: &str = "Proof_get_amount";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofGetAmountInput {}

pub type ProofGetAmountManifestInput = ProofGetAmountInput;

pub type ProofGetAmountOutput = Decimal;

pub const PROOF_GET_RESOURCE_ADDRESS_IDENT: &str = "Proof_get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofGetResourceAddressInput {}

pub type ProofGetResourceAddressManifestInput = ProofGetResourceAddressInput;

pub type ProofGetResourceAddressOutput = ResourceAddress;

pub const PROOF_CLONE_IDENT: &str = "clone";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofCloneInput {}

pub type ProofCloneManifestInput = ProofCloneInput;

pub type ProofCloneOutput = Proof;

//========
// Stub
//========

#[derive(Debug, PartialEq, Eq, Hash)]
#[must_use]
pub struct Proof(pub Own);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
#[must_use]
pub struct FungibleProof(pub Proof);

impl AsRef<Proof> for FungibleProof {
    fn as_ref(&self) -> &Proof {
        &self.0
    }
}

impl From<FungibleProof> for Proof {
    fn from(value: FungibleProof) -> Self {
        value.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
#[must_use]
pub struct NonFungibleProof(pub Proof);

impl AsRef<Proof> for NonFungibleProof {
    fn as_ref(&self) -> &Proof {
        &self.0
    }
}

impl From<NonFungibleProof> for Proof {
    fn from(value: NonFungibleProof) -> Self {
        value.0
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Proof {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Proof {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Proof {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for Proof {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_PROOF_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::own_proof_type_data()
    }
}

impl Describe<ScryptoCustomTypeKind> for FungibleProof {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_FUNGIBLE_PROOF_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::own_fungible_proof_type_data()
    }
}

impl Describe<ScryptoCustomTypeKind> for NonFungibleProof {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_NON_FUNGIBLE_PROOF_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::own_non_fungible_proof_type_data()
    }
}
