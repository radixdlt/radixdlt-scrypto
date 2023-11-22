(module
    ;; below line is invalid if feature 'Import/Export mutable globals' is disabled
    ;; see: https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md
    ;; It is also invalid from the Radix Engine point of view - this import is not included in the list of the valid imports
    (global $imported_global_mutable_value (import "env" "imported_global_mutable_value") (mut i32))

    ;; Simple function that returns imported global value in SBOR
    (func $Test_f (param $0 i64) (result i64)

        ;; Encode () in SBOR at address 0x0
        (i32.const 0)
        (i32.const 92)  ;; prefix
        (i32.store8)

        (i32.const 1)
        (i32.const 4)  ;; i32 value kind
        (i32.store8)

        (i32.const 2)
        (global.get $imported_global_mutable_value)     ;; Store the imported global value
        (i32.store8)

        ;; Return slice (ptr = 0, len = 6 (1 prefix + 1 value kind + 4 i32 len))
        (i64.const 6)
    )
    (memory $0 1)
    (export "memory" (memory $0))
    (export "Test_f" (func $Test_f))
)
