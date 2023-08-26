use rand::distributions::uniform::SampleUniform;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use radix_engine::blueprints::pool::one_resource_pool::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine::transaction::{BalanceChange, TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::pool::*;
use scrypto_unit::*;
use transaction::prelude::*;

struct ResourceTestFuzzer {
    rng: ChaCha8Rng,
}

impl ResourceTestFuzzer {
    fn new() -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);
        Self {
            rng,
        }
    }

    fn next_amount(&mut self) -> Decimal {
        let next_amount_type = self.rng.gen_range(0u32..6u32);
        match next_amount_type {
            0 => Decimal::ZERO,
            1 => Decimal::ONE,
            2 => Decimal::MAX,
            3 => Decimal::MIN,
            4 => Decimal(I192::ONE),
            _ => {
                let mut bytes = [0u8; 24];
                self.rng.fill_bytes(&mut bytes);
                Decimal(I192::from_le_bytes(&bytes))
            }
        }
    }
}

#[test]
fn run() {
    let mut test_runner = TestEnvironment::new(18);
    let mut fuzzer = ResourceTestFuzzer::new();

    for _ in 0..1000 {
        let contribute_amount = fuzzer.next_amount();
        test_runner.contribute(contribute_amount, true);

        let pool_unit_amount = fuzzer.next_amount();
        test_runner.call_get_redemption_value(pool_unit_amount, true);
    }
}

fn is_pool_emitter(event_type_identifier: &EventTypeIdentifier) -> bool {
    match event_type_identifier.0 {
        Emitter::Method(node_id, ObjectModuleId::Main) => match node_id.entity_type() {
            Some(
                EntityType::GlobalOneResourcePool
                | EntityType::GlobalTwoResourcePool
                | EntityType::GlobalMultiResourcePool,
            ) => true,
            _ => false,
        },
        _ => false,
    }
}

//===================================
// Test Runner and Utility Functions
//===================================

struct TestEnvironment {
    test_runner: DefaultTestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl TestEnvironment {
    fn new(divisibility: u8) -> Self {
        Self::new_with_owner(divisibility, OwnerRole::None)
    }

    fn new_account() {

    }

    fn new_with_owner(divisibility: u8, owner_role: OwnerRole) -> Self {
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility,
            account,
        );

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
                    OneResourcePoolInstantiateManifestInput {
                        resource_address,
                        pool_manager_rule: rule!(require(virtual_signature_badge)),
                        owner_role,
                        address_reservation: None,
                    },
                )
                .build();
            let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
            let commit_result = receipt.expect_commit_success();

            (
                commit_result.new_component_addresses()[0],
                commit_result.new_resource_addresses()[0],
            )
        };

        Self {
            test_runner,
            pool_component_address: pool_component,
            pool_unit_resource_address: pool_unit_resource,
            resource_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn contribute<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .mint_fungible(self.resource_address, amount.into())
            .take_all_from_worktop(self.resource_address, "contribution")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    OneResourcePoolContributeManifestInput {
                        bucket: lookup.bucket("contribution"),
                    },
                )
            })
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn redeem<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                self.account_component_address,
                self.pool_unit_resource_address,
                amount.into(),
            )
            .take_all_from_worktop(self.pool_unit_resource_address, "pool_unit")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_REDEEM_IDENT,
                    OneResourcePoolRedeemManifestInput {
                        bucket: lookup.bucket("pool_unit"),
                    },
                )
            })
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_deposit<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .mint_fungible(self.resource_address, amount.into())
            .take_all_from_worktop(self.resource_address, "to_deposit")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    OneResourcePoolProtectedDepositManifestInput {
                        bucket: lookup.bucket("to_deposit"),
                    },
                )
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        amount: D,
        withdraw_strategy: WithdrawStrategy,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                OneResourcePoolProtectedWithdrawManifestInput {
                    amount: amount.into(),
                    withdraw_strategy,
                },
            )
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> Decimal {
        let receipt = self.call_get_redemption_value(amount_of_pool_units, sign);
        receipt.expect_commit_success().output(1)
    }

    fn call_get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                OneResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                },
            )
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn get_vault_amount(&mut self, sign: bool) -> Decimal {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT,
                OneResourcePoolGetVaultAmountManifestInput,
            )
            .build();
        let receipt = self.execute_manifest(manifest, sign);
        receipt.expect_commit_success().output(1)
    }

    fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        sign: bool,
    ) -> TransactionReceipt {
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, self.initial_proofs(sign))
    }

    fn virtual_signature_badge(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::from_public_key(&self.account_public_key)
    }

    fn initial_proofs(&self, sign: bool) -> Vec<NonFungibleGlobalId> {
        if sign {
            vec![self.virtual_signature_badge()]
        } else {
            vec![]
        }
    }
}

fn is_one_resource_pool_does_non_fungible_resources_are_not_accepted(
    runtime_error: &RuntimeError,
) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::OneResourcePoolError(
            OneResourcePoolError::NonFungibleResourcesAreNotAccepted { .. }
        ))
    )
}
