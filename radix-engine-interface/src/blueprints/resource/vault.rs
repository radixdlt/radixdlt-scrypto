use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use bitflags::bitflags;
use radix_engine_common::data::scrypto::*;
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

pub const VAULT_TAKE_ADVANCED_IDENT: &str = "take_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultTakeAdvancedInput {
    pub amount: Decimal,
    pub withdraw_strategy: WithdrawStrategy,
}

pub type VaultTakeAdvancedOutput = Bucket;

pub const VAULT_GET_AMOUNT_IDENT: &str = "get_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultGetAmountInput {}

pub type VaultGetAmountOutput = Decimal;

pub const VAULT_RECALL_IDENT: &str = "recall";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VaultRecallInput {
    pub amount: Decimal,
}

pub type VaultRecallOutput = Bucket;

bitflags! {
    #[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
    #[derive(Sbor)]
    pub struct VaultFreezeFlags: u32 {
        const WITHDRAW = 0b00000001;
        const DEPOSIT = 0b00000010;
        const BURN = 0b00000100;
    }
}

pub const VAULT_FREEZE_IDENT: &str = "freeze";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VaultFreezeInput {
    pub to_freeze: VaultFreezeFlags,
}

pub type VaultFreezeOutput = ();

pub const VAULT_UNFREEZE_IDENT: &str = "unfreeze";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VaultUnfreezeInput {
    pub to_unfreeze: VaultFreezeFlags,
}

pub type VaultUnfreezeOutput = ();

pub const VAULT_BURN_IDENT: &str = "burn";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultBurnInput {
    pub amount: Decimal,
}

pub type VaultBurnOutput = ();

//========
// Stub
//========

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
#[must_use]
pub struct Vault(pub Own);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
#[must_use]
pub struct FungibleVault(pub Vault);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
#[must_use]
pub struct NonFungibleVault(pub Vault);

impl From<FungibleVault> for Vault {
    fn from(value: FungibleVault) -> Self {
        value.0
    }
}

impl From<NonFungibleVault> for Vault {
    fn from(value: NonFungibleVault) -> Self {
        value.0
    }
}

//========
// binary
//========

impl Describe<ScryptoCustomTypeKind> for Vault {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(well_known_scrypto_custom_types::OWN_VAULT_ID);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        well_known_scrypto_custom_types::own_vault_type_data()
    }
}

impl Describe<ScryptoCustomTypeKind> for FungibleVault {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(well_known_scrypto_custom_types::OWN_FUNGIBLE_VAULT_ID);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        well_known_scrypto_custom_types::own_fungible_vault_type_data()
    }
}

impl Describe<ScryptoCustomTypeKind> for NonFungibleVault {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(well_known_scrypto_custom_types::OWN_NON_FUNGIBLE_VAULT_ID);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        well_known_scrypto_custom_types::own_non_fungible_vault_type_data()
    }
}
