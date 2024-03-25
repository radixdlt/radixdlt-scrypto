use crate::blueprints::account::*;
use crate::blueprints::component::*;
use crate::blueprints::macros::*;
use crate::blueprints::resource::*;
use radix_common::data::manifest::model::*;
use radix_common::prelude::*;
use radix_common::*;

define_type_info_marker!(Some(LOCKER_PACKAGE), AccountLocker);

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
    output: type Global<AccountLockerObjectTypeInfo>,
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
        allow_forceful_withdraws: bool
    },
    output: type Global<AccountLockerObjectTypeInfo>,
    manifest_input: struct {
        allow_forceful_withdraws: bool
    }
}

//================
// Storer Methods
//================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: store,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        bucket: Bucket
    },
    output: type (),
    manifest_input: struct {
        claimant: ComponentAddress,
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: store_batch,
    input: struct {
        claimants: IndexMap<Global<AccountObjectTypeInfo>, ResourceSpecifier>,
        bucket: Bucket
    },
    output: type Option<Bucket>,
    manifest_input: struct {
        claimants: IndexMap<ComponentAddress, ResourceSpecifier>,
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: send_or_store,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        bucket: Bucket
    },
    output: type (),
    manifest_input: struct {
        claimant: ComponentAddress,
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: send_or_store_batch,
    input: struct {
        claimants: IndexMap<Global<AccountObjectTypeInfo>, ResourceSpecifier>,
        bucket: Bucket
    },
    output: type Option<Bucket>,
    manifest_input: struct {
        claimants: IndexMap<ComponentAddress, ResourceSpecifier>,
        bucket: ManifestBucket
    }
}

//===================
// Recoverer Methods
//===================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: recover,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        resource_address: ResourceAddress,
        amount: Decimal
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ComponentAddress,
        resource_address: ResourceAddress,
        amount: Decimal
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: recover_non_fungibles,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        resource_address: ResourceAddress,
        amount: IndexSet<NonFungibleLocalId>
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ComponentAddress,
        resource_address: ResourceAddress,
        amount: IndexSet<NonFungibleLocalId>
    }
}

//=====================
// Public User Methods
//=====================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: claim,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        resource_address: ResourceAddress,
        amount: Decimal
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ComponentAddress,
        resource_address: ResourceAddress,
        amount: Decimal
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: claim_non_fungibles,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        resource_address: ResourceAddress,
        amount: IndexSet<NonFungibleLocalId>
    },
    output: type Bucket,
    manifest_input: struct {
        claimant: ComponentAddress,
        resource_address: ResourceAddress,
        amount: IndexSet<NonFungibleLocalId>
    }
}

//================
// Getter Methods
//================

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: get_amount,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        resource_address: ResourceAddress,
    },
    output: type Decimal,
    manifest_input: struct {
        claimant: ComponentAddress,
        resource_address: ResourceAddress,
    }
}

define_invocation! {
    blueprint_name: AccountLocker,
    function_name: get_non_fungible_local_ids,
    input: struct {
        claimant: Global<AccountObjectTypeInfo>,
        resource_address: ResourceAddress,
        limit: u32
    },
    output: type IndexSet<NonFungibleLocalId>,
    manifest_input: struct {
        claimant: ComponentAddress,
        resource_address: ResourceAddress,
        limit: u32
    }
}

//==================
// Additional Types
//==================

#[derive(Clone, Debug, ScryptoSbor, ManifestSbor, PartialEq, Eq)]
pub enum ResourceSpecifier {
    Fungible(ResourceAddress, Decimal),
    NonFungible(ResourceAddress, IndexSet<NonFungibleLocalId>),
}
