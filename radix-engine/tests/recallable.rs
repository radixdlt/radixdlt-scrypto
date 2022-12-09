use radix_engine::engine::{
    AuthError, KernelError, ModuleError, REActor, RejectionError, ResolvedMethod, ResolvedReceiver,
    RuntimeError,
};
use radix_engine::model::MethodAuthorizationError;
use radix_engine::types::*;
use radix_engine_interface::api::types::{NativeMethod, RENodeId};
use radix_engine_interface::data::*;
use radix_engine_interface::node::NetworkDefinition;
use scrypto_unit::*;
use std::ops::Sub;
use transaction::builder::ManifestBuilder;
use transaction::model::BasicInstruction;

#[test]
fn non_existing_vault_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (_, _, account) = test_runner.new_allocated_account();

    let non_existing_vault_id = [0; 36];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .add_instruction(BasicInstruction::CallNativeMethod {
            method_ident: NativeMethodIdent {
                receiver: RENodeId::Vault(non_existing_vault_id),
                method_name: "recall".to_string(),
            },
            args: scrypto_encode(&VaultTakeInvocation {
                receiver: non_existing_vault_id,
                amount: Decimal::one(),
            })
            .unwrap(),
        })
        .0
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_rejection(|e| {
        e.eq(&RejectionError::ErrorBeforeFeeLoanRepaid(
            RuntimeError::KernelError(KernelError::RENodeNotFound(RENodeId::Vault(
                non_existing_vault_id,
            ))),
        ))
    });
}

#[test]
fn cannot_get_amount_on_direct_vault_access() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (_, _, account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_fungible_resource(10u32.into(), 0u8, account);
    let vaults = test_runner.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .add_instruction(BasicInstruction::CallNativeMethod {
            method_ident: NativeMethodIdent {
                receiver: RENodeId::Vault(vault_id),
                method_name: "get_amount".to_string(),
            },
            args: scrypto_encode(&VaultGetAmountInvocation { receiver: vault_id }).unwrap(),
        })
        .0
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        e.eq(&RuntimeError::ModuleError(ModuleError::AuthError(
            AuthError::VisibilityError(RENodeId::Vault(vault_id)),
        )))
    });
}

#[test]
fn cannot_take_on_non_recallable_vault() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (_, _, account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_fungible_resource(10u32.into(), 0u8, account);
    let vaults = test_runner.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .add_instruction(BasicInstruction::CallNativeMethod {
            method_ident: NativeMethodIdent {
                receiver: RENodeId::Vault(vault_id),
                method_name: "recall".to_string(),
            },
            args: scrypto_encode(&VaultTakeInvocation {
                receiver: vault_id,
                amount: Decimal::one(),
            })
            .unwrap(),
        })
        .0
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized {
                actor: REActor::Method(
                    ResolvedMethod::Native(NativeMethod::Vault(VaultMethod::Recall)),
                    ResolvedReceiver {
                        receiver: RENodeId::Vault(..),
                        ..
                    }
                ),
                error: MethodAuthorizationError::NotAuthorized,
                ..
            },))
        )
    });
}

#[test]
fn can_take_on_recallable_vault() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (_, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let recallable_token = test_runner.create_recallable_token(account);
    let vaults = test_runner.get_component_vaults(account, recallable_token);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .add_instruction(BasicInstruction::CallNativeMethod {
            method_ident: NativeMethodIdent {
                receiver: RENodeId::Vault(vault_id),
                method_name: "recall".to_string(),
            },
            args: scrypto_encode(&VaultTakeInvocation {
                receiver: vault_id,
                amount: Decimal::one(),
            })
            .unwrap(),
        })
        .0
        .call_method(
            other_account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    let original_account_amount = test_runner
        .get_component_resources(account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    let mut expected_amount: Decimal = 5u32.into();
    expected_amount = expected_amount.sub(Decimal::one());
    assert_eq!(expected_amount, original_account_amount);

    let other_amount = test_runner
        .get_component_resources(other_account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    assert_eq!(other_amount, Decimal::one());
}
