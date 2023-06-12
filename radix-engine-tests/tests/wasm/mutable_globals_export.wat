(module
    ;; below line is invalid if feature 'Import/Export mutable globals' is disabled
    ;; see: https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md
    (global $exported_global_mutable_value (export "exported_global_mutable_value") (mut i32) (i32.const 2222))

    ;; Simple function that sets global value and returns `()`
    (func $Test_f (param $0 i64) (result i64)

        (global.set $exported_global_mutable_value
            (i32.const 1111))

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
    (memory $0 1)
    (export "memory" (memory $0))
    (export "Test_f" (func $Test_f))
)
