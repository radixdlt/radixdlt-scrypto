use radix_engine::errors::{ApplicationError, RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_queries::typed_substate_layout::AccountError;
use scrypto_unit::{DefaultTestRunner, TestRunnerBuilder};
use transaction::prelude::*;
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
fn changing_default_deposit_rule_is_callable_with_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt =
            test_runner.transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn changing_default_deposit_rule_is_not_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut test_runner = AccountDepositModesTestRunner::new(is_virtual);

        // Act
        let receipt =
            test_runner.transition_default_deposit_rule(DefaultDepositRule::AllowExisting, false);

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
            .remove_resource_preference(resource_address, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        test_runner
            .add_to_deny_list(XRD, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        test_runner
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();
        test_runner
            .remove_resource_preference(resource_address, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::Reject, true)
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
            .transition_default_deposit_rule(DefaultDepositRule::Reject, true)
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
    test_runner: DefaultTestRunner,
    public_key: PublicKey,
    component_address: ComponentAddress,
}

impl AccountDepositModesTestRunner {
    pub fn new(virtual_account: bool) -> Self {
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
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
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 1)
            .take_all_from_worktop(resource_address, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                deposit_method.call(builder, self.component_address, bucket)
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn free_tokens_from_faucet_to_account(
        &mut self,
        deposit_method: DepositMethod,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "free_tokens")
            .with_bucket("free_tokens", |builder, bucket| {
                deposit_method.call(builder, self.component_address, bucket)
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn transition_default_deposit_rule(
        &mut self,
        default: DefaultDepositRule,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.component_address,
                ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                AccountSetDefaultDepositRuleInput { default },
            )
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn set_resource_preference(
        &mut self,
        resource_address: ResourceAddress,
        resource_preference: ResourcePreference,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.component_address,
                ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                AccountSetResourcePreferenceInput {
                    resource_address,
                    resource_preference,
                },
            )
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn remove_resource_preference(
        &mut self,
        resource_address: ResourceAddress,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.component_address,
                ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT,
                AccountRemoveResourcePreferenceInput { resource_address },
            )
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn add_to_allow_list(
        &mut self,
        resource_address: ResourceAddress,
        sign: bool,
    ) -> TransactionReceipt {
        self.set_resource_preference(resource_address, ResourcePreference::Allowed, sign)
    }

    pub fn add_to_deny_list(
        &mut self,
        resource_address: ResourceAddress,
        sign: bool,
    ) -> TransactionReceipt {
        self.set_resource_preference(resource_address, ResourcePreference::Disallowed, sign)
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
            .get_component_balance(self.component_address, resource_address);
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(self.component_address, resource_address, balance)
            .try_deposit_batch_or_refund(virtual_account, None)
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

impl DepositMethod {
    pub fn call(
        &self,
        manifest_builder: ManifestBuilder,
        account: ComponentAddress,
        bucket: ManifestBucket,
    ) -> ManifestBuilder {
        let (method, is_vec, insert_badge) = match self {
            Self::Deposit => (ACCOUNT_DEPOSIT_IDENT, false, false),
            Self::TryDeposit => (ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT, false, true),
            Self::TryDepositOrAbort => (ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT, false, true),
            Self::DepositBatch => (ACCOUNT_DEPOSIT_BATCH_IDENT, true, false),
            Self::TryDepositBatchOrRefund => {
                (ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT, true, true)
            }
            Self::TryDepositBatchOrAbort => (ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT, true, true),
        };

        let args = match (is_vec, insert_badge) {
            (true, true) => {
                manifest_args!(vec![bucket], Option::<ResourceOrNonFungible>::None)
            }
            (true, false) => manifest_args!(vec![bucket]),
            (false, true) => manifest_args!(bucket, Option::<ResourceOrNonFungible>::None),
            (false, false) => manifest_args!(bucket),
        };

        manifest_builder.call_method(account, method, args)
    }
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
