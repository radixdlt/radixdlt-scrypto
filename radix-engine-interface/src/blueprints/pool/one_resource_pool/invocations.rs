use crate::blueprints::macros::*;
use crate::blueprints::resource::*;
use radix_engine_common::data::manifest::model::*;
use radix_engine_common::math::*;
use radix_engine_common::prelude::*;
use radix_engine_common::*;

define_invocation! {
    blueprint_name: OneResourcePool,
    function_name: instantiate,
    input: struct {
        resource_address: ResourceAddress,
        pool_manager_rule: AccessRule
    },
    output: type ComponentAddress,
    manifest_input: struct {
        resource_address: ResourceAddress,
        pool_manager_rule: AccessRule
    }
}

define_invocation! {
    blueprint_name: OneResourcePool,
    function_name: contribute,
    input: struct {
        bucket: Bucket
    },
    output: type Bucket,
    manifest_input: struct {
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: OneResourcePool,
    function_name: redeem,
    input: struct {
        bucket: Bucket
    },
    output: type Bucket,
    manifest_input: struct {
        bucket: ManifestBucket
    }
}

define_invocation! {
    blueprint_name: OneResourcePool,
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
    blueprint_name: OneResourcePool,
    function_name: protected_withdraw,
    input: struct {
        amount: Decimal
    },
    output: type Bucket,
    manifest_input: struct {
        amount: Decimal
    }
}

define_invocation! {
    blueprint_name: OneResourcePool,
    function_name: get_redemption_value,
    input: struct {
        amount_of_pool_units: Decimal
    },
    output: type Decimal,
    manifest_input: struct {
        amount_of_pool_units: Decimal
    }
}

define_invocation! {
    blueprint_name: OneResourcePool,
    function_name: get_vault_amount,
    input: struct {},
    output: type Decimal,
    manifest_input: struct {}
}
