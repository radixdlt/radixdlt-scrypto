use radix_engine_tests::prelude::*;

#[test]
fn before_dugong_assert_access_rule_still_works_with_auth_module_disabled() {
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::CuttlefishPart2))
        .build();
    assert_access_rule_deny_all_with_no_auth_context(&mut ledger)
        .expect_commit_failure_containing_error("AssertAccessRuleFailed");
}

#[test]
fn after_dugong_assert_access_rule_is_no_op_with_auth_module_disabled() {
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Dugong))
        .build();
    assert_access_rule_deny_all_with_no_auth_context(&mut ledger).expect_commit_success();
}

fn assert_access_rule_deny_all_with_no_auth_context<E: NativeVmExtension, D: TestDatabase>(
    ledger: &mut LedgerSimulator<E, D>,
) -> TransactionReceipt {
    let assert_access_rule_component_address = {
        let package_address = ledger.publish_package_simple(PackageLoader::get("role_assignment"));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "AssertAccessRule", "new", manifest_args!())
            .build();

        let receipt = ledger.execute_manifest(manifest, []);
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            assert_access_rule_component_address,
            "assert_access_rule",
            (rule!(deny_all),),
        )
        .build();

    ledger.execute_manifest_with_execution_config(
        manifest,
        [],
        ExecutionConfig::for_preview_no_auth(NetworkDefinition::simulator()),
    )
}
