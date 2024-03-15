use radix_engine::errors::{ApplicationError, RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine_common::prelude::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_queries::typed_substate_layout::AccountError;
use radix_transactions::prelude::*;
use scrypto_test::prelude::{DefaultLedgerSimulator, LedgerSimulatorBuilder};

#[test]
fn account_deposit_method_is_callable_with_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt = ledger.free_tokens_from_faucet_to_account(DepositMethod::Deposit, true);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_deposit_batch_method_is_callable_with_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt = ledger.free_tokens_from_faucet_to_account(DepositMethod::DepositBatch, true);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_deposit_method_is_not_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt = ledger.free_tokens_from_faucet_to_account(DepositMethod::Deposit, false);

        // Assert
        receipt.expect_specific_failure(is_auth_unauthorized_error);
    }
}

#[test]
fn account_deposit_batch_method_is_not_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt = ledger.free_tokens_from_faucet_to_account(DepositMethod::DepositBatch, false);

        // Assert
        receipt.expect_specific_failure(is_auth_unauthorized_error);
    }
}

#[test]
fn account_try_deposit_method_is_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt = ledger.free_tokens_from_faucet_to_account(DepositMethod::TryDeposit, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_try_deposit_batch_or_refund_method_is_callable_without_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt = ledger
            .free_tokens_from_faucet_to_account(DepositMethod::TryDepositBatchOrRefund, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_try_deposit_batch_or_refund_method_is_callable_with_array_of_resources() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account_address) = ledger.new_account(true);

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "xrd_1a")
            .take_all_from_worktop(XRD, "xrd_1b")
            .try_deposit_batch_or_refund(account_address, ["xrd_1a", "xrd_1b"], None)
            .try_deposit_batch_or_refund(account_address, Vec::<String>::new(), None)
            .take_all_from_worktop(XRD, "xrd_2a")
            .take_all_from_worktop(XRD, "xrd_2b")
            .try_deposit_batch_or_abort(account_address, ["xrd_2a", "xrd_2b"], None)
            .try_deposit_batch_or_abort(account_address, Vec::<String>::new(), None)
            .build(),
        [],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn account_try_deposit_or_abort_method_is_callable_without_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt =
            ledger.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn account_try_deposit_batch_or_abort_method_is_callable_without_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt =
            ledger.free_tokens_from_faucet_to_account(DepositMethod::TryDepositBatchOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn changing_default_deposit_rule_is_callable_with_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt =
            ledger.transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn changing_default_deposit_rule_is_not_callable_with_out_owner_signature() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        // Act
        let receipt =
            ledger.transition_default_deposit_rule(DefaultDepositRule::AllowExisting, false);

        // Assert
        receipt.expect_specific_failure(is_auth_unauthorized_error);
    }
}

#[test]
fn allow_all_allows_for_all_resource_deposits() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_all_disallows_deposit_of_resource_in_deny_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();
        ledger
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn resource_in_deny_list_could_be_converted_to_resource_in_allow_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();
        ledger
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();
        ledger
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn resource_in_deny_list_could_be_removed_from_there() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();
        ledger
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();
        ledger
            .remove_resource_preference(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_disallows_deposit_of_resources_on_deny_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        ledger
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        ledger.add_to_deny_list(XRD, true).expect_commit_success();

        // Act
        let receipt =
            ledger.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn allow_existing_allows_deposit_of_xrd_if_not_on_deny_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        ledger
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_allows_deposit_of_an_existing_resource() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        let resource_address = ledger.freely_mintable_resource();
        ledger
            .mint_and_deposit(resource_address, DepositMethod::Deposit, true)
            .expect_commit_success();

        ledger
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_allows_deposit_of_an_existing_resource_even_if_account_has_none_of_it() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);

        let resource_address = ledger.freely_mintable_resource();
        ledger
            .mint_and_deposit(resource_address, DepositMethod::Deposit, true)
            .expect_commit_success();
        ledger.burn(resource_address);

        ledger
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn allow_existing_allows_deposit_of_a_resource_account_does_not_have_if_it_is_on_the_allow_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();
        ledger
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        ledger
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn removing_an_address_from_the_allow_list_removes_it() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();
        ledger
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        ledger
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();
        ledger
            .remove_resource_preference(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn transitioning_an_address_to_deny_list_works_as_expected() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();
        ledger
            .transition_default_deposit_rule(DefaultDepositRule::AllowExisting, true)
            .expect_commit_success();
        ledger
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();
        ledger
            .add_to_deny_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn disallow_all_does_not_permit_deposit_of_any_resource() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        ledger
            .transition_default_deposit_rule(DefaultDepositRule::Reject, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.free_tokens_from_faucet_to_account(DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_specific_failure(is_account_deposit_not_allowed_error);
    }
}

#[test]
fn disallow_all_permits_deposit_of_resource_in_allow_list() {
    // Arrange
    for is_virtual in [true, false] {
        let mut ledger = AccountDepositModesLedgerSimulator::new(is_virtual);
        let resource_address = ledger.freely_mintable_resource();
        ledger
            .transition_default_deposit_rule(DefaultDepositRule::Reject, true)
            .expect_commit_success();
        ledger
            .add_to_allow_list(resource_address, true)
            .expect_commit_success();

        // Act
        let receipt =
            ledger.mint_and_deposit(resource_address, DepositMethod::TryDepositOrAbort, false);

        // Assert
        receipt.expect_commit_success();
    }
}

struct AccountDepositModesLedgerSimulator {
    ledger: DefaultLedgerSimulator,
    public_key: PublicKey,
    component_address: ComponentAddress,
}

impl AccountDepositModesLedgerSimulator {
    pub fn new(virtual_account: bool) -> Self {
        let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
        let (public_key, _, component_address) = ledger.new_account(virtual_account);

        Self {
            component_address,
            public_key: public_key.into(),
            ledger,
        }
    }

    pub fn mint_and_deposit(
        &mut self,
        resource_address: ResourceAddress,
        deposit_method: DepositMethod,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
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
            .lock_fee_from_faucet()
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
            .lock_fee_from_faucet()
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
            .lock_fee_from_faucet()
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
            .lock_fee_from_faucet()
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
        self.ledger
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
            .ledger
            .get_component_balance(self.component_address, resource_address);
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.component_address, resource_address, balance)
            .try_deposit_entire_worktop_or_refund(virtual_account, None)
            .build();

        self.execute_manifest(manifest, true)
            .expect_commit_success();
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        sign: bool,
    ) -> TransactionReceipt {
        self.ledger.execute_manifest(
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
