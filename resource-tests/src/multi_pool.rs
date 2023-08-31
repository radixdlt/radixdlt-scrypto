use crate::TestFuzzer;
use radix_engine::types::FromRepr;
use radix_engine_common::constants::XRD;
use radix_engine_common::manifest_args;
use radix_engine_common::prelude::{ComponentAddress, Decimal, ManifestExpression, NonFungibleLocalId, VALIDATOR_OWNER_BADGE};
use radix_engine_common::types::ResourceAddress;
use radix_engine_interface::blueprints::pool::{OneResourcePoolContributeManifestInput, OneResourcePoolGetRedemptionValueManifestInput, OneResourcePoolProtectedDepositManifestInput, OneResourcePoolProtectedWithdrawManifestInput, OneResourcePoolRedeemManifestInput, ONE_RESOURCE_POOL_CONTRIBUTE_IDENT, ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT, ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT, ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT, ONE_RESOURCE_POOL_REDEEM_IDENT, TWO_RESOURCE_POOL_CONTRIBUTE_IDENT, TwoResourcePoolContributeManifestInput, TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT, TwoResourcePoolProtectedDepositManifestInput, TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT, TwoResourcePoolProtectedWithdrawManifestInput, TWO_RESOURCE_POOL_REDEEM_IDENT, TwoResourcePoolRedeemManifestInput, MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT, MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT, MultiResourcePoolProtectedDepositManifestInput, MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT, MultiResourcePoolProtectedWithdrawManifestInput, MULTI_RESOURCE_POOL_REDEEM_IDENT, MultiResourcePoolRedeemManifestInput, MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT, MultiResourcePoolGetRedemptionValueManifestInput};
use radix_engine_interface::data::manifest::ManifestArgs;
use transaction::builder::ManifestBuilder;
use utils::btreeset;

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
        fuzzer: &mut TestFuzzer,
        account_address: ComponentAddress,
        pool_address: ComponentAddress,
        pool_unit_resource_address: ResourceAddress,
        pool_resources: &Vec<ResourceAddress>,
    ) -> (ManifestBuilder, bool) {
        match self {
            MultiPoolFuzzAction::Contribute => {
                let resource_to_amount_mapping: Vec<(ResourceAddress, Decimal)> = pool_resources
                    .iter()
                    .map(|resource| (*resource, fuzzer.next_amount()))
                    .collect();

                let mut builder = ManifestBuilder::new();
                for (resource_address, amount) in resource_to_amount_mapping.iter() {
                    builder = builder.mint_fungible(*resource_address, *amount)
                }
                let builder = builder
                    .call_method(
                        pool_address,
                        MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                        manifest_args!(ManifestExpression::EntireWorktop),
                    );

                (builder, false)
            }
            MultiPoolFuzzAction::ProtectedDeposit => {
                let resource_address = pool_resources
                    .get(fuzzer.next_usize(pool_resources.len()))
                    .unwrap().clone();
                let amount = fuzzer.next_amount();

                let builder = builder
                    .mint_fungible(resource_address, amount)
                    .take_all_from_worktop(resource_address, "to_deposit")
                    .with_name_lookup(|builder, lookup| {
                        let bucket = lookup.bucket("to_deposit");
                        builder.call_method(
                            pool_address,
                            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                            MultiResourcePoolProtectedDepositManifestInput { bucket },
                        )
                    });

                (builder, amount.is_zero())
            }
            MultiPoolFuzzAction::ProtectedWithdraw => {
                let resource_address = pool_resources
                    .get(fuzzer.next_usize(pool_resources.len()))
                    .unwrap().clone();
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();

                let builder = builder
                    .call_method(
                        pool_address,
                        MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                        MultiResourcePoolProtectedWithdrawManifestInput {
                            resource_address,
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
                        pool_unit_resource_address,
                        amount,
                    )
                    .take_all_from_worktop(pool_unit_resource_address, "pool_units")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            pool_address,
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

                let builder = builder
                    .call_method(
                        pool_address,
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
