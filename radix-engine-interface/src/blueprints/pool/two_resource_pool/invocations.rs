use crate::blueprints::component::*;
use crate::blueprints::macros::*;
use crate::blueprints::resource::*;
use radix_common::data::manifest::model::*;
use radix_common::math::*;
use radix_common::prelude::*;

pub const TWO_RESOURCE_POOL_BLUEPRINT: &str = "TwoResourcePool";

define_type_marker!(Some(POOL_PACKAGE), TwoResourcePool);

define_invocation! {
    blueprint_name: TwoResourcePool,
    function_name: instantiate,
    input: struct {
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        resource_addresses: (ResourceAddress, ResourceAddress),
        address_reservation: Option<GlobalAddressReservation>
    },
    output: type Global<TwoResourcePoolMarker>,
    manifest_input: struct {
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        resource_addresses: (ManifestResourceAddress, ManifestResourceAddress),
        address_reservation: Option<ManifestAddressReservation>
    }
}

define_invocation! {
    blueprint_name: TwoResourcePool,
    function_name: contribute,
    input: struct {
        buckets: (Bucket, Bucket)
    },
    output: type (Bucket, Option<Bucket>),
    manifest_input: struct {
        buckets: (ManifestBucket, ManifestBucket)
    }
}

define_invocation! {
    blueprint_name: TwoResourcePool,
    function_name: redeem,
    input: struct {
        bucket: Bucket
    },
    output: type (Bucket, Bucket),
    manifest_input: struct {
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: TwoResourcePool,
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
    blueprint_name: TwoResourcePool,
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
    blueprint_name: TwoResourcePool,
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
    blueprint_name: TwoResourcePool,
    function_name: get_vault_amounts,
    input: struct {},
    output: type IndexMap<ResourceAddress, Decimal>,
    manifest_input: struct {}
}
