use crate::blueprints::account::*;
use crate::blueprints::component::*;
use crate::blueprints::macros::*;
use crate::blueprints::resource::*;
use radix_common::data::manifest::model::*;
use radix_common::prelude::*;

define_type_marker!(Some(LOCKER_PACKAGE), AccountLocker);

pub const ACCOUNT_LOCKER_BLUEPRINT: &str = "AccountLocker";

//===========
// Functions
//===========

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: instantiate,
    input: struct {
        owner_role: OwnerRole,
        storer_role: AccessRule,
        storer_updater_role: AccessRule,
        recoverer_role: AccessRule,
        recoverer_updater_role: AccessRule,
        address_reservation: Option<GlobalAddressReservation>
    },
    output: type Global<AccountLockerMarker>,
    manifest_input: struct {
        owner_role: OwnerRole,
        storer_role: AccessRule,
        storer_updater_role: AccessRule,
        recoverer_role: AccessRule,
        recoverer_updater_role: AccessRule,
        address_reservation: Option<ManifestAddressReservation>
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: instantiate_simple,
    input: struct {
        allow_recover: bool
    },
    output: type (Global<AccountLockerMarker>, Bucket),
    manifest_input: struct {
        allow_recover: bool
    }
}

//================
// Storer Methods
//================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: store,
    input: struct {
        claimant: Global<AccountMarker>,
        bucket: Bucket,
        try_direct_send: bool
    },
    output: type (),
    manifest_input: struct {
        claimant: ManifestComponentAddress,
        bucket: ManifestBucket,
        try_direct_send: bool
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: airdrop,
    input: struct {
        claimants: IndexMap<Global<AccountMarker>, ResourceSpecifier>,
        bucket: Bucket,
        try_direct_send: bool
    },
    output: type Option<Bucket>,
    manifest_input: struct {
        claimants: IndexMap<ManifestComponentAddress, ResourceSpecifier>,
        bucket: ManifestBucket,
        try_direct_send: bool
    }
}

//===================
// Recoverer Methods
//===================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: recover,
    input: struct {
        claimant: Global<AccountMarker>,
        resource_address: ResourceAddress,
        amount: Decimal
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ManifestComponentAddress,
        resource_address: ManifestResourceAddress,
        amount: Decimal
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: recover_non_fungibles,
    input: struct {
        claimant: Global<AccountMarker>,
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ManifestComponentAddress,
        resource_address: ManifestResourceAddress,
        ids: IndexSet<NonFungibleLocalId>
    }
}

//=====================
// Public User Methods
//=====================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: claim,
    input: struct {
        claimant: Global<AccountMarker>,
        resource_address: ResourceAddress,
        amount: Decimal
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ManifestComponentAddress,
        resource_address: ManifestResourceAddress,
        amount: Decimal
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: claim_non_fungibles,
    input: struct {
        claimant: Global<AccountMarker>,
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ManifestComponentAddress,
        resource_address: ManifestResourceAddress,
        ids: IndexSet<NonFungibleLocalId>
    }
}

//================
// Getter Methods
//================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: get_amount,
    input: struct {
        claimant: Global<AccountMarker>,
        resource_address: ResourceAddress,
    },
    output: type Decimal,
    manifest_input: struct {
        claimant: DynamicComponentAddress,
        resource_address: DynamicResourceAddress,
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: get_non_fungible_local_ids,
    input: struct {
        claimant: Global<AccountMarker>,
        resource_address: ResourceAddress,
        limit: u32
    },
    output: type IndexSet<NonFungibleLocalId>,
    manifest_input: struct {
        claimant: ManifestComponentAddress,
        resource_address: ManifestResourceAddress,
        limit: u32
    }
}

//==================
// Additional Types
//==================

#[derive(Clone, Debug, ScryptoSbor, ManifestSbor, PartialEq, Eq)]
pub enum ResourceSpecifier {
    Fungible(Decimal),
    NonFungible(IndexSet<NonFungibleLocalId>),
}

impl ResourceSpecifier {
    pub fn new_empty(resource_address: ResourceAddress) -> Self {
        if resource_address.is_fungible() {
            Self::Fungible(Default::default())
        } else {
            Self::NonFungible(Default::default())
        }
    }

    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            (ResourceSpecifier::Fungible(amount1), ResourceSpecifier::Fungible(amount2)) => amount1
                .checked_add(*amount2)
                .map(ResourceSpecifier::Fungible),
            (ResourceSpecifier::NonFungible(ids1), ResourceSpecifier::NonFungible(ids2)) => Some(
                ResourceSpecifier::NonFungible(ids1.clone().union(ids2).cloned().collect()),
            ),
            (ResourceSpecifier::Fungible(_), ResourceSpecifier::NonFungible(_))
            | (ResourceSpecifier::NonFungible(_), ResourceSpecifier::Fungible(_)) => None,
        }
    }

    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            (ResourceSpecifier::Fungible(amount1), ResourceSpecifier::Fungible(amount2)) => {
                // Ensure that amount 2 is smaller than or equal to amount 1
                if amount2 <= amount1 {
                    amount1.checked_sub(*amount2).map(Self::Fungible)
                } else {
                    None
                }
            }
            (ResourceSpecifier::NonFungible(ids1), ResourceSpecifier::NonFungible(ids2)) => {
                // Ensure that ids2 is a subset of ids1
                if ids2.is_subset(ids1) {
                    Some(Self::NonFungible(
                        ids1.clone().difference(ids2).cloned().collect(),
                    ))
                } else {
                    None
                }
            }
            (ResourceSpecifier::Fungible(_), ResourceSpecifier::NonFungible(_))
            | (ResourceSpecifier::NonFungible(_), ResourceSpecifier::Fungible(_)) => None,
        }
    }
}
