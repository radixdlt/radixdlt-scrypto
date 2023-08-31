use crate::TestFuzzer;
use radix_engine::types::FromRepr;
use radix_engine_common::constants::XRD;
use radix_engine_common::manifest_args;
use radix_engine_common::prelude::{ComponentAddress, NonFungibleLocalId, VALIDATOR_OWNER_BADGE};
use radix_engine_common::types::ResourceAddress;
use radix_engine_interface::blueprints::pool::{OneResourcePoolContributeManifestInput, OneResourcePoolGetRedemptionValueManifestInput, OneResourcePoolProtectedDepositManifestInput, OneResourcePoolProtectedWithdrawManifestInput, OneResourcePoolRedeemManifestInput, ONE_RESOURCE_POOL_CONTRIBUTE_IDENT, ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT, ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT, ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT, ONE_RESOURCE_POOL_REDEEM_IDENT, TWO_RESOURCE_POOL_CONTRIBUTE_IDENT, TwoResourcePoolContributeManifestInput, TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT, TwoResourcePoolProtectedDepositManifestInput, TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT, TwoResourcePoolProtectedWithdrawManifestInput, TWO_RESOURCE_POOL_REDEEM_IDENT, TwoResourcePoolRedeemManifestInput};
use radix_engine_interface::data::manifest::ManifestArgs;
use transaction::builder::ManifestBuilder;
use utils::btreeset;

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
        fuzzer: &mut TestFuzzer,
        account_address: ComponentAddress,
        pool_address: ComponentAddress,
        pool_unit_resource_address: ResourceAddress,
        resource_address1: ResourceAddress,
        resource_address2: ResourceAddress,
    ) -> (ManifestBuilder, bool) {
        match self {
            TwoPoolFuzzAction::Contribute => {
                let amount1 = fuzzer.next_amount();
                let amount2 = fuzzer.next_amount();

                let builder = builder
                    .mint_fungible(resource_address1, amount1)
                    .mint_fungible(resource_address2, amount2)
                    .take_all_from_worktop(resource_address1, "resource_1")
                    .take_all_from_worktop(resource_address2, "resource_2")
                    .with_name_lookup(|builder, lookup| {
                        let bucket1 = lookup.bucket("resource_1");
                        let bucket2 = lookup.bucket("resource_2");
                        builder.call_method(
                            pool_address,
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
                    .mint_fungible(resource_address1, amount)
                    .take_all_from_worktop(resource_address1, "deposit")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            pool_address,
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
                    .mint_fungible(resource_address2, amount)
                    .take_all_from_worktop(resource_address2, "deposit")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            pool_address,
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
                let builder = builder
                    .call_method(
                        pool_address,
                        TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        TwoResourcePoolProtectedWithdrawManifestInput {
                            resource_address: resource_address1,
                            amount: amount.into(),
                            withdraw_strategy,
                        },
                    );

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::ProtectedWithdraw2 => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();
                let builder = builder
                    .call_method(
                        pool_address,
                        TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        TwoResourcePoolProtectedWithdrawManifestInput {
                            resource_address: resource_address2,
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
                        pool_unit_resource_address,
                        amount,
                    )
                    .take_all_from_worktop(pool_unit_resource_address, "pool_units")
                    .with_name_lookup(|builder, lookup| {
                        let bucket = lookup.bucket("pool_units");
                        builder.call_method(
                            pool_address,
                            TWO_RESOURCE_POOL_REDEEM_IDENT,
                            TwoResourcePoolRedeemManifestInput { bucket },
                        )
                    });

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::GetRedemptionValue1 => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();

                let builder = builder
                    .call_method(
                        pool_address,
                        TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        TwoResourcePoolProtectedWithdrawManifestInput {
                            resource_address: resource_address1,
                            amount,
                            withdraw_strategy,
                        },
                    );

                (builder, amount.is_zero())
            }
            TwoPoolFuzzAction::GetRedemptionValue2 => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();

                let builder = builder
                    .call_method(
                        pool_address,
                        TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        TwoResourcePoolProtectedWithdrawManifestInput {
                            resource_address: resource_address2,
                            amount,
                            withdraw_strategy,
                        },
                    );

                (builder, amount.is_zero())
            }
        }
    }
}
