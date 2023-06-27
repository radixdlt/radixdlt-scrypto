use radix_engine::errors::{ApplicationError, RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_queries::typed_substate_layout::AccountError;
use scrypto_unit::TestRunner;
use transaction::builder::{ManifestBuilder, TransactionManifestV1};
use transaction::signing::secp256k1::Secp256k1PrivateKey;

#[test]
fn account_deposit_method_is_callable_with_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt = test_runner.free_tokens_from_faucet_to_account(DepositMethod::Deposit, true);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_deposit_batch_method_is_callable_with_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt =
            test_runner.free_tokens_from_faucet_to_account(DepositMethod::DepositBatch, true);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_deposit_method_is_not_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt = test_runner.free_tokens_from_faucet_to_account(DepositMethod::Deposit, false);

        // Assert
        receipt.expect_specific_failure(is_auth_unauthorized_error);
    }
}

#[test]
fn account_deposit_batch_method_is_not_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt =
            test_runner.free_tokens_from_faucet_to_account(DepositMethod::DepositBatch, false);

        // Assert
        receipt.expect_specific_failure(is_auth_unauthorized_error);
    }
}

#[test]
fn account_try_deposit_method_is_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt =
            test_runner.free_tokens_from_faucet_to_account(DepositMethod::TryDeposit, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_try_deposit_batch_or_refund_method_is_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt = test_runner
            .free_tokens_from_faucet_to_account(DepositMethod::TryDepositBatchOrRefund, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_try_deposit_or_abort_method_is_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt =
            test_runner.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_try_deposit_batch_or_abort_method_is_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt = test_runner
            .free_tokens_from_faucet_to_account(DepositMethod::TryDepositBatchOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn changing_account_default_deposit_rule_is_callable_with_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt = test_runner.transition_account_default_deposit_rule(
            AccountDefaultDepositRule::AllowExisting,
            true,
        );

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn changing_account_default_deposit_rule_is_not_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt = test_runner.transition_account_default_deposit_rule(
            AccountDefaultDepositRule::AllowExisting,
            false,
        );

        // Assert
        receipt.expect_specific_failure(is_auth_unauthorized_error);
    }
}

#[test]
fn allow_all_allows_for_all_resource_deposits() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_all_disallows_deposit_of_resource_in_deny_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn resource_in_deny_list_could_be_converted_to_resource_in_allow_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();
        test_runner
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn resource_in_deny_list_could_be_removed_from_there() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();
        test_runner
            .remove_from_deny_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_disallows_deposit_of_resources_on_deny_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        test_runner
            .add_to_deny_list(RADIX_TOKEN, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn allow_existing_allows_deposit_of_xrd_if_not_on_deny_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::AllowExisting, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_allows_deposit_of_an_existing_resource() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .mint_and_deposit(resource_address, DepositMethod::Deposit, true)
            .expect_commit_success();

        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::AllowExisting, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_allows_deposit_of_an_existing_resource_even_if_account_has_none_of_it() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .mint_and_deposit(resource_address, DepositMethod::Deposit, true)
            .expect_commit_success();
        test_runner.burn(resource_address);

        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::AllowExisting, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_allows_deposit_of_a_resource_account_does_not_have_if_it_is_on_the_allow_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        test_runner
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn removing_an_address_from_the_allow_list_removes_it() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        test_runner
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();
        test_runner
            .remove_from_allow_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn transitioning_an_address_to_deny_list_works_as_expected() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        test_runner
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();
        test_runner
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn disallow_all_does_not_permit_deposit_of_any_resource() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::Reject, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn disallow_all_permits_deposit_of_resource_in_allow_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);
        let resource_address = test_runner.freely_mintable_resource();
        test_runner
            .transition_account_default_deposit_rule(AccountDefaultDepositRule::Reject, true)
            .expect_commit_success();
        test_runner
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            test_runner.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

struct AccountDepositModesTestRunner {
    test_runner: TestRunner,
    public_key: PublicKey,
    component_address: ComponentAddress,
}

impl AccountDepositModesTestRunner {
    pub fn new(virtual_account: bool) -> Self {
        let mut test_runner = TestRunner::builder().without_trace().build();
        let (public_key, _, component_address) = test_runner.new_account(virtual_account);

        Self {
            component_address,
            public_key: public_key.into(),
            test_runner,
        }
    }

    pub fn mint_and_deposit(
        &mut self,
        resource_address: ResourceAddress,
        deposit_method: DepositMethod,
        sign: bool,
    ) -> TransactionReceipt {
        let (method, is_vec) = match deposit_method {
            DepositMethod::Deposit => (ACCOUNT_DEPOSIT_IDENT, false),
            DepositMethod::TryDeposit => (ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT, false),
            DepositMethod::TryDepositOrAbort => (ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT, false),
            DepositMethod::DepositBatch => (ACCOUNT_DEPOSIT_BATCH_IDENT, true),
            DepositMethod::TryDepositBatchOrRefund => {
                (ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT, true)
            }
            DepositMethod::TryDepositBatchOrAbort => {
                (ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT, true)
            }
        };

        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 1.into())
            .take_all_from_worktop(resource_address, |builder, bucket| {
                let args = if is_vec {
                    manifest_args!(vec![bucket])
                } else {
                    manifest_args!(bucket)
                };
                builder.call_method(self.component_address, method, args)
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn free_tokens_from_faucet_to_account(
        &mut self,
        deposit_method: DepositMethod,
        sign: bool,
    ) -> TransactionReceipt {
        let (method, is_vec) = match deposit_method {
            DepositMethod::Deposit => (ACCOUNT_DEPOSIT_IDENT, false),
            DepositMethod::TryDeposit => (ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT, false),
            DepositMethod::TryDepositOrAbort => (ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT, false),
            DepositMethod::DepositBatch => (ACCOUNT_DEPOSIT_BATCH_IDENT, true),
            DepositMethod::TryDepositBatchOrRefund => {
                (ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT, true)
            }
            DepositMethod::TryDepositBatchOrAbort => {
                (ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT, true)
            }
        };

        let manifest = ManifestBuilder::new()
            .call_method(FAUCET, "free", manifest_args!())
            .take_all_from_worktop(RADIX_TOKEN, |builder, bucket| {
                let args = if is_vec {
                    manifest_args!(vec![bucket])
                } else {
                    manifest_args!(bucket)
                };
                builder.call_method(self.component_address, method, args)
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn transition_account_default_deposit_rule(
        &mut self,
        default_deposit_rule: AccountDefaultDepositRule,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.component_address,
                ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT,
                to_manifest_value_and_unwrap!(&AccountChangeDefaultDepositRuleInput {
                    default_deposit_rule,
                }),
            )
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn configure_resource_deposit_rule(
        &mut self,
        resource_address: ResourceAddress,
        resource_deposit_configuration: ResourceDepositRule,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.component_address,
                ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT,
                to_manifest_value_and_unwrap!(&AccountConfigureResourceDepositRuleInput {
                    resource_address,
                    resource_deposit_configuration,
                }),
            )
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn add_to_allow_list(
        &mut self,
        resource_address: ResourceAddress,
        sign: bool,
    ) -> TransactionReceipt {
        self.configure_resource_deposit_rule(resource_address, ResourceDepositRule::Allowed, sign)
    }

    pub fn add_to_deny_list(
        &mut self,
        resource_address: ResourceAddress,
        sign: bool,
    ) -> TransactionReceipt {
        self.configure_resource_deposit_rule(
            resource_address,
            ResourceDepositRule::Disallowed,
            sign,
        )
    }

    pub fn remove_from_allow_list(
        &mut self,
        resource_address: ResourceAddress,
        sign: bool,
    ) -> TransactionReceipt {
        self.configure_resource_deposit_rule(resource_address, ResourceDepositRule::Neither, sign)
    }

    pub fn remove_from_deny_list(
        &mut self,
        resource_address: ResourceAddress,
        sign: bool,
    ) -> TransactionReceipt {
        self.configure_resource_deposit_rule(resource_address, ResourceDepositRule::Neither, sign)
    }

    pub fn virtual_signature_badge(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::from_public_key(&self.public_key)
    }

    pub fn freely_mintable_resource(&mut self) -> ResourceAddress {
        self.test_runner
            .create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                18,
                self.component_address,
            )
    }

    pub fn burn(&mut self, resource_address: ResourceAddress) {
        let virtual_account = ComponentAddress::virtual_account_from_public_key(
            &Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key(),
        );

        let balance = self
            .test_runner
            .account_balance(self.component_address, resource_address);
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(self.component_address, resource_address, balance.unwrap())
            .try_deposit_batch_or_refund(virtual_account)
            .build();

        self.execute_manifest(manifest, true)
            .expect_commit_success();
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        sign: bool,
    ) -> TransactionReceipt {
        self.test_runner.execute_manifest_ignoring_fee(
            manifest,
            if sign {
                vec![self.virtual_signature_badge()]
            } else {
                vec![]
            },
        )
    }
}

enum DepositMethod {
    Deposit,
    TryDeposit,
    TryDepositOrAbort,

    DepositBatch,
    TryDepositBatchOrRefund,
    TryDepositBatchOrAbort,
}

fn is_auth_unauthorized_error(runtime_error: &RuntimeError) -> bool {
    matches!(
        runtime_error,
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(..)))
    )
}

fn is_account_deposit_not_allowed_error(runtime_error: &RuntimeError) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::AccountError(
            AccountError::DepositIsDisallowed { .. }
        ))
    )
}
