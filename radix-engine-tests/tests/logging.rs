use radix_engine::{
    errors::{RuntimeError, SystemError},
    types::*,
};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_basic_transfer() {
    // Arrange
    let code = wat2wasm(
        r##"
(module
   (import "env" "emit_log" (func $emit_log (param i32 i32 i32 i32)))
   (data (i32.const 0) "\5C\22\01\00TEXT")
   (func $test (param $0 i64) (result i64)
        ;; create a local variable and initialize it to 0
        (local $i i32)

        (loop $my_loop

            ;; add one to $i
            local.get $i
            i32.const 1
            i32.add
            local.set $i

            ;; emit log
            (call $emit_log
                (i32.const 0)
                (i32.const 4)
                (i32.const 4)
                (i32.const 65536)
            )

            ;; if $i is less than 1000000000 branch to loop
            local.get $i
            i32.const 1000000000
            i32.lt_s
            br_if $my_loop
        )
        (i64.const 0)
   )
   (memory $0 64)
   (export "memory" (memory $0))
   (export "Test_f" (func $test))
)
    "##
        .replace("TEXT", " ".repeat(65536).as_str())
        .as_str(),
    );
    let mut test_runner = TestRunner::builder().without_trace().build();
    let package_address = test_runner.publish_package(
        code,
        single_function_package_definition("Test", "f"),
        BTreeMap::new(),
        OwnerRole::None,
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::SystemError(SystemError::TooManyLogs))
    })
}
