use crate::data::scrypto::model::Own;
use crate::data::scrypto::model::*;
use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::data::scrypto::ScryptoCustomValueKind;
use crate::math::*;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::*;

use super::Proof;

pub const BUCKET_BLUEPRINT: &str = "Bucket";

pub const BUCKET_BURN_IDENT: &str = "burn_bucket";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct BucketBurnInput {
    pub bucket: Bucket,
}

pub type BucketBurnOutput = ();

impl Clone for BucketBurnInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const BUCKET_DROP_EMPTY_IDENT: &str = "Bucket_drop_empty";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct BucketDropEmptyInput {
    pub bucket: Bucket,
}

pub type BucketDropEmptyOutput = ();

pub const BUCKET_TAKE_IDENT: &str = "Bucket_take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketTakeInput {
    pub amount: Decimal,
}

pub type BucketTakeOutput = Bucket;

pub const BUCKET_PUT_IDENT: &str = "Bucket_put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct BucketPutInput {
    pub bucket: Bucket,
}

pub type BucketPutOutput = ();

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

pub type BucketTakeNonFungiblesOutput = Bucket;

pub const BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT: &str = "Bucket_get_non_fungible_local_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetNonFungibleLocalIdsInput {}

pub type BucketGetNonFungibleLocalIdsOutput = BTreeSet<NonFungibleLocalId>;

pub const BUCKET_GET_AMOUNT_IDENT: &str = "Bucket_get_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetAmountInput {}

pub type BucketGetAmountOutput = Decimal;

pub const BUCKET_GET_RESOURCE_ADDRESS_IDENT: &str = "Bucket_get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetResourceAddressInput {}

pub type BucketGetResourceAddressOutput = ResourceAddress;

pub const BUCKET_CREATE_PROOF_IDENT: &str = "Bucket_create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketCreateProofInput {}

pub type BucketCreateProofOutput = Proof;

pub const BUCKET_LOCK_AMOUNT_IDENT: &str = "Bucket_lock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketLockAmountInput {
    pub amount: Decimal,
}

pub type BucketLockAmountOutput = ();

pub const BUCKET_UNLOCK_AMOUNT_IDENT: &str = "Bucket_unlock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketUnlockAmountInput {
    pub amount: Decimal,
}

pub type BucketUnlockAmountOutput = ();

pub const BUCKET_LOCK_NON_FUNGIBLES_IDENT: &str = "Bucket_lock_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketLockNonFungiblesInput {
    pub local_ids: BTreeSet<NonFungibleLocalId>,
}

pub type BucketLockNonFungiblesOutput = ();

pub const BUCKET_UNLOCK_NON_FUNGIBLES_IDENT: &str = "Bucket_unlock_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketUnlockNonFungiblesInput {
    pub local_ids: BTreeSet<NonFungibleLocalId>,
}

pub type BucketUnlockNonFungiblesOutput = ();

//========
// Stub
//========

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub Own);

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Bucket {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Bucket {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Bucket {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for Bucket {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(
        crate::data::scrypto::well_known_scrypto_custom_types::OWN_BUCKET_ID,
    );
}
