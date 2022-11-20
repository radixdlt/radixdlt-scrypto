use radix_engine::engine::{AuthError, KernelError, ModuleError, RejectionError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

#[test]
fn non_existing_vault_should_cause_error() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (_, _, account) = test_runner.new_allocated_account();

    let non_existing_vault_id = [0; 36];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .add_instruction(Instruction::CallNativeMethod {
            method_ident: NativeMethodIdent {
                receiver: RENodeId::Vault(non_existing_vault_id),
                method_name: "take".to_string(),
            },
            args: scrypto_encode(&VaultTakeInvocation {
                receiver: non_existing_vault_id,
                amount: Decimal::one(),
            }),
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
fn owned_vault_should_not_be_visible_to_manifest() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (_, _, account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_fungible_resource(10u32.into(), 0u8, account);
    let vaults = test_runner.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .add_instruction(Instruction::CallNativeMethod {
            method_ident: NativeMethodIdent {
                receiver: RENodeId::Vault(vault_id),
                method_name: "take".to_string(),
            },
            args: scrypto_encode(&VaultTakeInvocation {
                receiver: vault_id,
                amount: Decimal::one(),
            }),
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
        e.eq(&RuntimeError::ModuleError(ModuleError::AuthError(
            AuthError::VisibilityError(RENodeId::Vault(vault_id)),
        )))
    });
}
