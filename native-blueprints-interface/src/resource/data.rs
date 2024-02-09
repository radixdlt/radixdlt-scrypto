use super::WORKTOP_BLUEPRINT;
use crate::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::data::scrypto::model::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::math::*;
use radix_engine_common::prelude::*;
use radix_engine_common::{ManifestSbor, ScryptoSbor};

pub fn check_fungible_amount(amount: &Decimal, divisibility: u8) -> bool {
    !amount.is_negative()
        && amount.0 % I192::from(10i128.pow((18 - divisibility).into())) == I192::from(0)
}

pub fn check_non_fungible_amount(amount: &Decimal) -> Result<u32, ()> {
    // Integers between [0..u32::MAX]
    u32::try_from(amount).map_err(|_| ())
}

#[macro_export]
macro_rules! resource_roles {
    (
        $roles_struct:ident,
        $actor_field:ident,
        $updater_field:ident,
        $actor_field_name:expr,
        $updater_field_name:expr,
        $default_rule:expr
    ) => {
        #[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
        pub struct $roles_struct<T> {
            pub $actor_field: T,
            pub $updater_field: T,
        }

        impl $roles_struct<radix_engine_common::types::RoleDefinition> {
            pub fn to_role_init(self) -> radix_engine_common::prelude::RoleAssignmentInit {
                let mut roles = radix_engine_common::prelude::RoleAssignmentInit::new();
                roles.define_role($actor_field_name, self.$actor_field);
                roles.define_role($updater_field_name, self.$updater_field);
                roles
            }
        }

        impl Default for $roles_struct<radix_engine_common::types::RoleDefinition> {
            fn default() -> Self {
                Self {
                    $actor_field: Some($default_rule),
                    $updater_field: Some(AccessRule::DenyAll),
                }
            }
        }
    };
}

resource_roles!(
    MintRoles,
    minter,
    minter_updater,
    MINTER_ROLE,
    MINTER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! mint_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(radix_engine_common::internal_roles_struct!(MintRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    BurnRoles,
    burner,
    burner_updater,
    BURNER_ROLE,
    BURNER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! burn_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(radix_engine_common::internal_roles_struct!(BurnRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    RecallRoles,
    recaller,
    recaller_updater,
    RECALLER_ROLE,
    RECALLER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! recall_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(radix_engine_common::internal_roles_struct!(RecallRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    FreezeRoles,
    freezer,
    freezer_updater,
    FREEZER_ROLE,
    FREEZER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! freeze_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(radix_engine_common::internal_roles_struct!(FreezeRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    WithdrawRoles,
    withdrawer,
    withdrawer_updater,
    WITHDRAWER_ROLE,
    WITHDRAWER_UPDATER_ROLE,
    AccessRule::AllowAll
);
#[macro_export]
macro_rules! withdraw_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(radix_engine_common::internal_roles_struct!(WithdrawRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    DepositRoles,
    depositor,
    depositor_updater,
    DEPOSITOR_ROLE,
    DEPOSITOR_UPDATER_ROLE,
    AccessRule::AllowAll
);
#[macro_export]
macro_rules! deposit_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(radix_engine_common::internal_roles_struct!(DepositRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    NonFungibleDataUpdateRoles,
    non_fungible_data_updater,
    non_fungible_data_updater_updater,
    NON_FUNGIBLE_DATA_UPDATER_ROLE,
    NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE,
    AccessRule::DenyAll
);

#[macro_export]
macro_rules! non_fungible_data_update_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(radix_engine_common::internal_roles_struct!(NonFungibleDataUpdateRoles, $($role => $rule;)*))
    });
}

/// Define the withdraw strategy when request amount does not match underlying
/// resource divisibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum WithdrawStrategy {
    Exact,
    Rounded(RoundingMode),
}

pub trait ForWithdrawal {
    fn for_withdrawal(
        &self,
        divisibility: u8,
        withdraw_strategy: WithdrawStrategy,
    ) -> Option<Decimal>;
}

impl ForWithdrawal for Decimal {
    fn for_withdrawal(
        &self,
        divisibility: u8,
        withdraw_strategy: WithdrawStrategy,
    ) -> Option<Decimal> {
        match withdraw_strategy {
            WithdrawStrategy::Exact => Some(self.clone()),
            WithdrawStrategy::Rounded(mode) => self.checked_round(divisibility, mode),
        }
    }
}

impl Default for WithdrawStrategy {
    fn default() -> Self {
        Self::Exact
    }
}

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, Sbor, Eq, PartialEq)]
pub enum ResourceType {
    /// Represents a fungible resource
    Fungible { divisibility: u8 },

    /// Represents a non-fungible resource
    NonFungible { id_type: NonFungibleIdType },
}

impl ResourceType {
    pub fn divisibility(&self) -> Option<u8> {
        match self {
            ResourceType::Fungible { divisibility } => Some(*divisibility),
            ResourceType::NonFungible { .. } => None,
        }
    }

    pub fn id_type(&self) -> Option<NonFungibleIdType> {
        match self {
            ResourceType::Fungible { .. } => None,
            ResourceType::NonFungible { id_type } => Some(*id_type),
        }
    }

    pub fn is_fungible(&self) -> bool {
        match self {
            ResourceType::Fungible { .. } => true,
            ResourceType::NonFungible { .. } => false,
        }
    }

    pub fn check_amount(&self, amount: Decimal) -> bool {
        match self {
            ResourceType::Fungible { divisibility } => {
                check_fungible_amount(&amount, *divisibility)
            }
            ResourceType::NonFungible { .. } => check_non_fungible_amount(&amount).is_ok(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ResourceError {
    InsufficientBalance { requested: Decimal, actual: Decimal },
    InvalidTakeAmount,
    MissingNonFungibleLocalId(NonFungibleLocalId),
    DecimalOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct LiquidFungibleResource {
    /// The total amount.
    amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct VaultFrozenFlag {
    pub frozen: VaultFreezeFlags,
}

impl Default for VaultFrozenFlag {
    fn default() -> Self {
        Self {
            frozen: VaultFreezeFlags::empty(),
        }
    }
}

impl LiquidFungibleResource {
    pub fn new(amount: Decimal) -> Self {
        Self { amount }
    }

    pub fn default() -> Self {
        Self::new(Decimal::zero())
    }

    pub fn amount(&self) -> Decimal {
        self.amount.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.amount.is_zero()
    }

    pub fn put(&mut self, other: LiquidFungibleResource) {
        // update liquidity
        // NOTE: Decimal arithmetic operation safe unwrap.
        // Mint limit should prevent from overflowing
        self.amount = self.amount.checked_add(other.amount()).expect("Overflow");
    }

    pub fn take_by_amount(
        &mut self,
        amount_to_take: Decimal,
    ) -> Result<LiquidFungibleResource, ResourceError> {
        // deduct from liquidity pool
        if self.amount < amount_to_take {
            return Err(ResourceError::InsufficientBalance {
                requested: amount_to_take,
                actual: self.amount,
            });
        }
        self.amount = self
            .amount
            .checked_sub(amount_to_take)
            .ok_or(ResourceError::DecimalOverflow)?;
        Ok(LiquidFungibleResource::new(amount_to_take))
    }

    pub fn take_all(&mut self) -> LiquidFungibleResource {
        self.take_by_amount(self.amount())
            .expect("Take all from `Resource` should not fail")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LiquidNonFungibleResource {
    /// The total non-fungible ids.
    pub ids: IndexSet<NonFungibleLocalId>,
}

impl LiquidNonFungibleResource {
    pub fn new(ids: IndexSet<NonFungibleLocalId>) -> Self {
        Self { ids }
    }

    pub fn default() -> Self {
        Self::new(IndexSet::default())
    }

    pub fn ids(&self) -> &IndexSet<NonFungibleLocalId> {
        &self.ids
    }

    pub fn into_ids(self) -> IndexSet<NonFungibleLocalId> {
        self.ids
    }

    pub fn amount(&self) -> Decimal {
        self.ids.len().into()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn put(&mut self, other: LiquidNonFungibleResource) -> Result<(), ResourceError> {
        self.ids.extend(other.ids);
        Ok(())
    }

    pub fn take_by_amount(&mut self, n: u32) -> Result<LiquidNonFungibleResource, ResourceError> {
        if self.ids.len() < n as usize {
            return Err(ResourceError::InsufficientBalance {
                actual: Decimal::from(self.ids.len()),
                requested: Decimal::from(n),
            });
        }
        let ids: IndexSet<NonFungibleLocalId> = self.ids.iter().take(n as usize).cloned().collect();
        self.take_by_ids(&ids)
    }

    pub fn take_by_ids(
        &mut self,
        ids_to_take: &IndexSet<NonFungibleLocalId>,
    ) -> Result<LiquidNonFungibleResource, ResourceError> {
        for id in ids_to_take {
            if !self.ids.remove(id) {
                return Err(ResourceError::MissingNonFungibleLocalId(id.clone()));
            }
        }
        Ok(LiquidNonFungibleResource::new(ids_to_take.clone()))
    }

    pub fn take_all(&mut self) -> LiquidNonFungibleResource {
        LiquidNonFungibleResource {
            ids: core::mem::replace(&mut self.ids, indexset!()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LockedFungibleResource {
    /// The locked amounts and the corresponding times of being locked.
    pub amounts: IndexMap<Decimal, usize>,
}

impl LockedFungibleResource {
    pub fn default() -> Self {
        Self {
            amounts: index_map_new(),
        }
    }

    pub fn is_locked(&self) -> bool {
        !self.amounts.is_empty()
    }

    pub fn amount(&self) -> Decimal {
        let mut max = Decimal::ZERO;
        for amount in self.amounts.keys() {
            if amount > &max {
                max = amount.clone()
            }
        }
        max
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LockedNonFungibleResource {
    /// The locked non-fungible ids and the corresponding times of being locked.
    pub ids: IndexMap<NonFungibleLocalId, usize>,
}

impl LockedNonFungibleResource {
    pub fn default() -> Self {
        Self {
            ids: index_map_new(),
        }
    }

    pub fn is_locked(&self) -> bool {
        !self.ids.is_empty()
    }

    pub fn amount(&self) -> Decimal {
        self.ids.len().into()
    }

    pub fn ids(&self) -> IndexSet<NonFungibleLocalId> {
        self.ids.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct LiquidNonFungibleVault {
    pub amount: Decimal,
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
#[sbor(transparent)]
pub struct OwnedWorktop(pub Own);

impl Describe<ScryptoCustomTypeKind> for OwnedWorktop {
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1("OwnedWorktop".as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names("OwnedWorktop"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(Some(RESOURCE_PACKAGE), WORKTOP_BLUEPRINT.to_string()),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}
