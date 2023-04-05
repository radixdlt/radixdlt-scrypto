use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::data::scrypto::ScryptoCustomValueKind;
use crate::math::*;
use crate::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const VAULT_PUT_IDENT: &str = "put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct VaultPutInput {
    pub bucket: Bucket,
}

pub type VaultPutOutput = ();

impl Clone for VaultPutInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const VAULT_TAKE_IDENT: &str = "take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultTakeInput {
    pub amount: Decimal,
}

pub type VaultTakeOutput = Bucket;

pub const VAULT_RECALL_IDENT: &str = "recall";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultRecallInput {
    pub amount: Decimal,
}

pub type VaultRecallOutput = Bucket;

pub const VAULT_GET_AMOUNT_IDENT: &str = "get_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultGetAmountInput {}

pub type VaultGetAmountOutput = Decimal;

pub const VAULT_CREATE_PROOF_OF_ALL_IDENT: &str = "create_proof_of_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultCreateProofOfAllInput {}

pub type VaultCreateProofOfAllOutput = Proof;

pub const VAULT_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultCreateProofOfAmountInput {
    pub amount: Decimal,
}

pub type VaultCreateProofOfAmountOutput = Proof;

//========
// Stub
//========

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Vault(pub Own);

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Vault {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Vault {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Vault {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for Vault {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(
        crate::data::scrypto::well_known_scrypto_custom_types::OWN_VAULT_ID,
    );
}
