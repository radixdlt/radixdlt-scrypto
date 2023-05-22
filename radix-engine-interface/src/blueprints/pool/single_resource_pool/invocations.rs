use crate::blueprints::macros::*;
use crate::blueprints::resource::*;
use radix_engine_common::math::*;
use radix_engine_common::types::*;

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: instantiate,
    input: struct {
        resource_address: ResourceAddress
    },
    output: type Bucket
}

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: instantiate_with_owner_rule,
    input: struct {
        resource_address: ResourceAddress,
        owner_rule: AccessRule
    },
    output: type ()
}

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: contribute,
    input: struct {
        bucket: Bucket
    },
    output: type Bucket
}

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: redeem,
    input: struct {
        bucket: Bucket
    },
    output: type Bucket
}

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: protected_deposit,
    input: struct {
        bucket: Bucket
    },
    output: type ()
}

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: protected_withdraw,
    input: struct {
        amount: Decimal
    },
    output: type Bucket
}

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: get_redemption_value,
    input: struct {
        amount_of_pool_units: Decimal
    },
    output: type Decimal
}

define_invocation! {
    blueprint_name: SingleResourcePool,
    function_name: get_vault_amount,
    input: struct {},
    output: type Decimal
}
