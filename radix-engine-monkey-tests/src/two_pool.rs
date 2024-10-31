use crate::{SystemTestFuzzer, TwoPoolMeta};
use radix_common::prelude::ComponentAddress;
use radix_engine_interface::blueprints::pool::{
    TwoResourcePoolContributeManifestInput, TwoResourcePoolProtectedDepositManifestInput,
    TwoResourcePoolProtectedWithdrawManifestInput, TwoResourcePoolRedeemManifestInput,
    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT, TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
    TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT, TWO_RESOURCE_POOL_REDEEM_IDENT,
};
use radix_engine_interface::prelude::*;
use radix_transactions::builder::ManifestBuilder;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum TwoPoolFuzzAction {
    Contribute,
    ProtectedDeposit1,
    ProtectedDeposit2,
    ProtectedWithdraw1,
    ProtectedWithdraw2,
    Redeem,
    GetRedemptionValue1,
    GetRedemptionValue2,
}

impl TwoPoolFuzzAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        account_address: ComponentAddress,
        two_pool_meta: &TwoPoolMeta,
    ) -> (ManifestBuilder, bool) {
        match self {
            TwoPoolFuzzAction::Contribute => {
                let amount1 = fuzzer.next_amount();
                let amount2 = fuzzer.next_amount();

                let builder = builder
                    .mint_fungible(two_pool_meta.resource_address1, amount1)
                    .mint_fungible(two_pool_meta.resource_address2, amount2)
                    .take_all_from_worktop(two_pool_meta.resource_address1, "resource_1")
                    .take_all_from_worktop(two_pool_meta.resource_address2, "resource_2")
                    .with_name_lookup(|builder, lookup| {
                        let bucket1 = lookup.bucket("resource_1");
                        let bucket2 = lookup.bucket("resource_2");
                        builder.call_method(
                            two_pool_meta.pool_address,
                            TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                            TwoResourcePoolContributeManifestInput {
                                buckets: (bucket1, bucket2),
                            },
                        )
                    });

                (builder, amount1.is_zero() && amount2.is_zero())
            }
            TwoPoolFuzzAction::ProtectedDeposit1 => {
                let amount = fuzzer.next_amount();

                let builder = builder
                    .mint_fungible(two_pool_meta.resource_address1, amount)
                    .take_all_from_worktop(two_pool_meta.resource_address1, "deposit")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            two_pool_meta.pool_address,
                            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                            TwoResourcePoolProtectedDepositManifestInput {
                                bucket: lookup.bucket("deposit"),
                            },
                        )
                    });

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::ProtectedDeposit2 => {
                let amount = fuzzer.next_amount();

                let builder = builder
                    .mint_fungible(two_pool_meta.resource_address2, amount)
                    .take_all_from_worktop(two_pool_meta.resource_address2, "deposit")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            two_pool_meta.pool_address,
                            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                            TwoResourcePoolProtectedDepositManifestInput {
                                bucket: lookup.bucket("deposit"),
                            },
                        )
                    });

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::ProtectedWithdraw1 => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();
                let builder = builder.call_method(
                    two_pool_meta.pool_address,
                    TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                    TwoResourcePoolProtectedWithdrawManifestInput {
                        resource_address: two_pool_meta.resource_address1.into(),
                        amount: amount.into(),
                        withdraw_strategy,
                    },
                );

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::ProtectedWithdraw2 => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();
                let builder = builder.call_method(
                    two_pool_meta.pool_address,
                    TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                    TwoResourcePoolProtectedWithdrawManifestInput {
                        resource_address: two_pool_meta.resource_address2.into(),
                        amount,
                        withdraw_strategy,
                    },
                );

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::Redeem => {
                let amount = fuzzer.next_amount();

                let builder = builder
                    .withdraw_from_account(
                        account_address,
                        two_pool_meta.pool_unit_resource_address,
                        amount,
                    )
                    .take_all_from_worktop(two_pool_meta.pool_unit_resource_address, "pool_units")
                    .with_name_lookup(|builder, lookup| {
                        let bucket = lookup.bucket("pool_units");
                        builder.call_method(
                            two_pool_meta.pool_address,
                            TWO_RESOURCE_POOL_REDEEM_IDENT,
                            TwoResourcePoolRedeemManifestInput { bucket },
                        )
                    });

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::GetRedemptionValue1 => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();

                let builder = builder.call_method(
                    two_pool_meta.pool_address,
                    TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                    TwoResourcePoolProtectedWithdrawManifestInput {
                        resource_address: two_pool_meta.resource_address1.into(),
                        amount,
                        withdraw_strategy,
                    },
                );

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::GetRedemptionValue2 => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();

                let builder = builder.call_method(
                    two_pool_meta.pool_address,
                    TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                    TwoResourcePoolProtectedWithdrawManifestInput {
                        resource_address: two_pool_meta.resource_address2.into(),
                        amount,
                        withdraw_strategy,
                    },
                );

                (builder, amount.is_zero())
            }
        }
    }
}
