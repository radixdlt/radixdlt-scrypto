use crate::{MultiPoolMeta, SystemTestFuzzer};
use radix_common::data::manifest::ManifestArgs;
use radix_common::manifest_args;
use radix_common::prelude::{ComponentAddress, Decimal, ManifestExpression};
use radix_common::types::ResourceAddress;
use radix_engine_interface::blueprints::pool::{
    MultiResourcePoolGetRedemptionValueManifestInput,
    MultiResourcePoolProtectedDepositManifestInput,
    MultiResourcePoolProtectedWithdrawManifestInput, MultiResourcePoolRedeemManifestInput,
    MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT, MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
    MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT, MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
    MULTI_RESOURCE_POOL_REDEEM_IDENT,
};
use radix_engine_interface::prelude::*;
use radix_transactions::builder::ManifestBuilder;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum MultiPoolFuzzAction {
    Contribute,
    ProtectedDeposit,
    ProtectedWithdraw,
    Redeem,
    GetRedemptionValue,
}

impl MultiPoolFuzzAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        account_address: ComponentAddress,
        multi_pool_meta: &MultiPoolMeta,
    ) -> (ManifestBuilder, bool) {
        match self {
            MultiPoolFuzzAction::Contribute => {
                let resource_to_amount_mapping: Vec<(ResourceAddress, Decimal)> = multi_pool_meta
                    .pool_resources
                    .iter()
                    .map(|resource| (*resource, fuzzer.next_amount()))
                    .collect();

                let mut builder = ManifestBuilder::new();
                for (resource_address, amount) in resource_to_amount_mapping.iter() {
                    builder = builder.mint_fungible(*resource_address, *amount)
                }
                let builder = builder.call_method(
                    multi_pool_meta.pool_address,
                    MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    manifest_args!(ManifestExpression::EntireWorktop),
                );

                (builder, false)
            }
            MultiPoolFuzzAction::ProtectedDeposit => {
                let resource_address = multi_pool_meta
                    .pool_resources
                    .get(fuzzer.next_usize(multi_pool_meta.pool_resources.len()))
                    .unwrap()
                    .clone();
                let amount = fuzzer.next_amount();

                let builder = builder
                    .mint_fungible(resource_address, amount)
                    .take_all_from_worktop(resource_address, "to_deposit")
                    .with_name_lookup(|builder, lookup| {
                        let bucket = lookup.bucket("to_deposit");
                        builder.call_method(
                            multi_pool_meta.pool_address,
                            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                            MultiResourcePoolProtectedDepositManifestInput { bucket },
                        )
                    });

                (builder, amount.is_zero())
            }
            MultiPoolFuzzAction::ProtectedWithdraw => {
                let resource_address = multi_pool_meta
                    .pool_resources
                    .get(fuzzer.next_usize(multi_pool_meta.pool_resources.len()))
                    .unwrap()
                    .clone();
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();

                let builder = builder.call_method(
                    multi_pool_meta.pool_address,
                    MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                    MultiResourcePoolProtectedWithdrawManifestInput {
                        resource_address: resource_address.into(),
                        amount,
                        withdraw_strategy,
                    },
                );

                (builder, amount.is_zero())
            }
            MultiPoolFuzzAction::Redeem => {
                let amount = fuzzer.next_amount();

                let builder = builder
                    .withdraw_from_account(
                        account_address,
                        multi_pool_meta.pool_unit_resource_address,
                        amount,
                    )
                    .take_all_from_worktop(multi_pool_meta.pool_unit_resource_address, "pool_units")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            multi_pool_meta.pool_address,
                            MULTI_RESOURCE_POOL_REDEEM_IDENT,
                            MultiResourcePoolRedeemManifestInput {
                                bucket: lookup.bucket("pool_units"),
                            },
                        )
                    });

                (builder, amount.is_zero())
            }
            MultiPoolFuzzAction::GetRedemptionValue => {
                let amount = fuzzer.next_amount();

                let builder = builder.call_method(
                    multi_pool_meta.pool_address,
                    MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                    MultiResourcePoolGetRedemptionValueManifestInput {
                        amount_of_pool_units: amount,
                    },
                );

                (builder, amount.is_zero())
            }
        }
    }
}
