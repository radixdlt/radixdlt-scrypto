use crate::blueprints::component::*;
use crate::blueprints::macros::*;
use crate::blueprints::resource::*;
use radix_common::data::manifest::model::*;
use radix_common::math::*;
use radix_common::prelude::*;

pub const MULTI_RESOURCE_POOL_BLUEPRINT: &str = "MultiResourcePool";

define_type_marker!(Some(POOL_PACKAGE), MultiResourcePool);

define_invocation! {
    blueprint_name: MultiResourcePool,
    function_name: instantiate,
    input: struct {
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        resource_addresses: IndexSet<ResourceAddress>,
        address_reservation: Option<GlobalAddressReservation>
    },
    output: type Global<MultiResourcePoolMarker>,
    manifest_input: struct {
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        resource_addresses: IndexSet<ManifestResourceAddress>,
        address_reservation: Option<ManifestAddressReservation>
    }
}

define_invocation! {
    blueprint_name: MultiResourcePool,
    function_name: contribute,
    input: struct {
        buckets: Vec<Bucket>
    },
    output: type (Bucket, Vec<Bucket>),
    manifest_input: struct {
        buckets: ManifestBucketBatch
    }
}

define_invocation! {
    blueprint_name: MultiResourcePool,
    function_name: redeem,
    input: struct {
        bucket: Bucket
    },
    output: type Vec<Bucket>,
    manifest_input: struct {
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: MultiResourcePool,
    function_name: protected_deposit,
    input: struct {
        bucket: Bucket
    },
    output: type (),
    manifest_input: struct {
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: MultiResourcePool,
    function_name: protected_withdraw,
    input: struct {
        resource_address: ResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy
    },
    output: type Bucket,
    manifest_input: struct {
        resource_address: ManifestResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy
    }
}

define_invocation! {
    blueprint_name: MultiResourcePool,
    function_name: get_redemption_value,
    input: struct {
        amount_of_pool_units: Decimal
    },
    output: type IndexMap<ResourceAddress, Decimal>,
    manifest_input: struct {
        amount_of_pool_units: Decimal
    }
}

define_invocation! {
    blueprint_name: MultiResourcePool,
    function_name: get_vault_amounts,
    input: struct {},
    output: type IndexMap<ResourceAddress, Decimal>,
    manifest_input: struct {}
}
