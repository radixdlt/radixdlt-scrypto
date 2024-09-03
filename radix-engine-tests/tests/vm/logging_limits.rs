use radix_engine::{
    errors::{RuntimeError, SystemModuleError},
    system::system_modules::limits::TransactionLimitsError,
};
use radix_engine_interface::prelude::*;
use scrypto_test::prelude::*;

fn prepare_code(message_size: usize, iterations: usize) -> Vec<u8> {
    wat2wasm(
        r##"
(module
   (import "env" "sys_log" (func $sys_log (param i32 i32 i32 i32)))
   (data (i32.const 0) "\5C\22\01\00TEXT_CONTENT")
   (func $test (param $0 i64) (result i64)
        ;; create a local variable and initialize it to 0
        (local $i i32)

        (loop $my_loop

            ;; add one to $i
            local.get $i
            i32.const 1
            i32.add
            local.set $i

            ;; sys log
            (call $sys_log
                (i32.const 0)
                (i32.const 4)
                (i32.const 4)
                (i32.const TEXT_SIZE)
            )

            ;; if $i is less than ITERATIONS branch to loop
            local.get $i
            i32.const ITERATIONS
            i32.lt_s
            br_if $my_loop
        )
        
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
   (memory $0 64)
   (export "memory" (memory $0))
   (export "Test_f" (func $test))
)
    "##
        .replace("TEXT_CONTENT", " ".repeat(message_size).as_str())
        .replace("TEXT_SIZE", message_size.to_string().as_str())
        .replace("ITERATIONS", iterations.to_string().as_str())
        .as_str(),
    )
}

fn test_emit_log(message_size: usize, iterations: usize, expected_err: Option<RuntimeError>) {
    // Arrange
    let code = prepare_code(message_size, iterations);
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();

    // Assert
    let receipt = ledger.execute_manifest(manifest, vec![]);
    if let Some(e) = expected_err {
        receipt.expect_specific_failure(|x| x.eq(&e));
    } else {
        receipt.expect_commit_success();
    }
}

#[test]
fn test_emit_some_logs() {
    test_emit_log(MAX_LOG_SIZE, MAX_NUMBER_OF_LOGS - 1, None);
}

#[test]
fn test_emit_large_logs() {
    test_emit_log(
        MAX_LOG_SIZE + 1,
        1,
        Some(RuntimeError::SystemModuleError(
            SystemModuleError::TransactionLimitsError(TransactionLimitsError::LogSizeTooLarge {
                actual: MAX_LOG_SIZE + 1,
                max: MAX_LOG_SIZE,
            }),
        )),
    );
}
#[test]
fn test_emit_lots_of_logs() {
    test_emit_log(
        MAX_LOG_SIZE,
        1_000_000,
        Some(RuntimeError::SystemModuleError(
            SystemModuleError::TransactionLimitsError(TransactionLimitsError::TooManyLogs),
        )),
    );
}
