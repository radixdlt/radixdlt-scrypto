use radix_engine::types::*;
use radix_engine::vm::wasm::DEFAULT_WASM_ENGINE_CACHE_SIZE;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

/// Long running test which verifies that the Wasm cache is properly evicting entries
/// Ignored for day-to-day unit testing as it takes a long while to execute
#[test]
#[ignore]
fn publishing_many_packages_should_not_cause_system_failure() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let code = wat2wasm(&format!(
        r#"
                (module
                    (data (i32.const 0) "{}")
                    (memory $0 64)
                    (func $Test_f (param $0 i64) (result i64)
                      ;; Encode () in SBOR at address 0x0
                      (i32.const 0)
                      (i32.const 92)  ;; prefix
                      (i32.store8)
                      (i32.const 1)
                      (i32.const 33)  ;; tuple value kind
                      (i32.store8)
                      (i32.const 2)
                      (i32.const 0)  ;; tuple length
                      (i32.store8)

                      ;; Return slice (ptr = 0, len = 3)
                      (i64.const 3)
                    )
                    (export "memory" (memory $0))
                    (export "Test_f" (func $Test_f))
                )
            "#,
        "i".repeat(DEFAULT_MAX_INVOKE_INPUT_SIZE - 1024)
    ));

    // Act
    for _ in 0..(DEFAULT_WASM_ENGINE_CACHE_SIZE + 200) {
        let manifest = ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .publish_package_advanced(
                None,
                code.clone(),
                single_function_package_definition("Test", "f"),
                BTreeMap::new(),
                OwnerRole::None,
            )
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        let result = receipt.expect_commit_success();
        let package_address = result.new_package_addresses()[0];

        let manifest = ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .call_function(package_address, "Test", "f", manifest_args!())
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }
}
