(module
    ;; below line is invalid if feature 'Import/Export mutable globals' is disabled
    ;; see: https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md
    (global $g (import "env" "global_mutable_value") (mut i32))

    ;; Simple function that always returns `0`
    (func $increase_global_value (param $step i32) (result i32)

        (global.set $g
            (i32.add
                (global.get $g)
                (local.get $step)))

        (i32.const 0)
    )
    (memory $0 1)
    (export "memory" (memory $0))
    (export "increase_global_value" (func $increase_global_value))
    (export "global_mutable_value" (global $g))
)
