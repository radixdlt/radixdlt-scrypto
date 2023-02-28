use crate::abi::*;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::model::Own;
use crate::data::ScryptoCustomTypeKind;
use crate::data::ScryptoCustomValueKind;
use crate::math::*;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub const BUCKET_BLUEPRINT: &str = "Bucket";

pub const BUCKET_DROP_EMPTY_IDENT: &str = "Bucket_drop_empty";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct BucketDropEmptyInput {
    pub bucket: Bucket,
}

pub const BUCKET_TAKE_IDENT: &str = "Bucket_take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketTakeInput {
    pub amount: Decimal,
}

pub const BUCKET_PUT_IDENT: &str = "Bucket_put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct BucketPutInput {
    pub bucket: Bucket,
}

impl Clone for BucketPutInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const BUCKET_TAKE_NON_FUNGIBLES_IDENT: &str = "Bucket_take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketTakeNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub const BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT: &str = "Bucket_get_non_fungible_local_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetNonFungibleLocalIdsInput {}

pub const BUCKET_GET_AMOUNT_IDENT: &str = "Bucket_get_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetAmountInput {}

pub const BUCKET_GET_RESOURCE_ADDRESS_IDENT: &str = "Bucket_get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetResourceAddressInput {}

pub const BUCKET_CREATE_PROOF_IDENT: &str = "Bucket_create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketCreateProofInput {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketLockAmountInput {
    pub amount: Decimal,
}

pub const BUCKET_LOCK_AMOUNT_IDENT: &str = "Bucket_lock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketUnlockAmountInput {
    pub amount: Decimal,
}

pub const BUCKET_UNLOCK_AMOUNT_IDENT: &str = "Bucket_unlock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketLockNonFungiblesInput {
    pub local_ids: BTreeSet<NonFungibleLocalId>,
}

pub const BUCKET_LOCK_NON_FUNGIBLES_IDENT: &str = "Bucket_lock_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketUnlockNonFungiblesInput {
    pub local_ids: BTreeSet<NonFungibleLocalId>,
}

pub const BUCKET_UNLOCK_NON_FUNGIBLES_IDENT: &str = "Bucket_unlock_non_fungibles";

//========
// Stub
//========

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub BucketId);

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Bucket {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Bucket {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Own::Bucket(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Bucket {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::Bucket(bucket_id) => Ok(Self(bucket_id)),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for Bucket {
    fn describe() -> scrypto_abi::Type {
        Type::Bucket
    }
}

impl Describe<ScryptoCustomTypeKind<GlobalTypeId>> for Bucket {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(crate::data::well_known_scrypto_custom_types::OWN_BUCKET_ID);
}
