use radix_common::*;
use radix_common::constants::*;
use radix_common::prelude::*;
use radix_engine::{system::system_type_checker::TypeCheckError, updates::ProtocolUpdates};
use radix_engine::errors::*;
use radix_engine_interface::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_queries::typed_substate_layout::*;
use radix_transactions::builder::*;
use scrypto_test::ledger_simulator::*;

#[test]
fn get_owner_role_method_call_fails_without_the_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .without_kernel_trace()
        .with_custom_protocol_updates(ProtocolUpdates::none())
        .build();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_role_assignment_method(
                FAUCET,
                ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT,
                RoleAssignmentGetOwnerRoleInput,
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        error
            == &RuntimeError::SystemError(SystemError::TypeCheckError(
            TypeCheckError::BlueprintPayloadDoesNotExist(
                Box::new(BlueprintInfo {
                    blueprint_id: BlueprintId {
                        package_address: ROLE_ASSIGNMENT_MODULE_PACKAGE,
                        blueprint_name: ROLE_ASSIGNMENT_BLUEPRINT.to_owned(),
                    },
                    blueprint_version: Default::default(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: Default::default(),
                    generic_substitutions: Default::default(),
                }),
                BlueprintPayloadIdentifier::Function(
                    ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT.to_owned(),
                    InputOrOutput::Input,
                ),
            ),
        ))
    });
}

#[test]
fn get_owner_role_method_call_succeeds_with_the_protocol_update1() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_role_assignment_method(
                FAUCET,
                ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT,
                RoleAssignmentGetOwnerRoleInput,
            )
            .build(),
        vec![],
    );

    // Assert
    let owner_role_entry = receipt.expect_commit_success().output::<OwnerRoleEntry>(1);
    assert_eq!(
        owner_role_entry,
        OwnerRoleEntry {
            rule: rule!(deny_all),
            updater: OwnerRoleUpdater::None,
        }
    )
}

#[test]
fn get_owner_role_method_call_succeeds_with_the_protocol_update2() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let account =
        ledger.new_account_advanced(OwnerRole::Fixed(rule!(require_amount(dec!(100), XRD))));

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_role_assignment_method(
                account,
                ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT,
                RoleAssignmentGetOwnerRoleInput,
            )
            .build(),
        vec![],
    );

    // Assert
    let owner_role_entry = receipt.expect_commit_success().output::<OwnerRoleEntry>(1);
    assert_eq!(
        owner_role_entry,
        OwnerRoleEntry {
            rule: rule!(require_amount(dec!(100), XRD)),
            updater: OwnerRoleUpdater::None,
        }
    )
}
