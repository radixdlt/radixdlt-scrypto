(module
    (import "env" "test_host_read_memory" (func $host_read_memory (param i32 i32)))
    (import "env" "test_host_write_memory" (func $host_write_memory (param i32 i32)))
    (import "env" "test_host_check_memory_is_clean" (func $host_check_memory_is_clean (result i64)))

    (func $Test_grow_memory (param $pages i64)  (result i64)
        ;; grow memory by given pages cnt
        (memory.grow
            (i32.wrap_i64
                (local.get $pages)
            )
        )
        ;; drop operand from stack put by memory grow
        drop

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

    (func $Test_read_memory (param $offs i64) (param $len i64) (result i64)
        ;; read_memory
        (call $host_read_memory
            (i32.wrap_i64
                (local.get $offs)
            )
            (i32.wrap_i64
                (local.get $len)
            )
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

    (func $Test_write_memory (param $offs i64) (param $len i64) (result i64)
        ;; write_memory
        (call $host_write_memory
            (i32.wrap_i64
                (local.get $offs)
            )
            (i32.wrap_i64
                (local.get $len)
            )
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

    (func $Test_check_memory_is_clean (result i64)
        (local $clean i64)

        (call $host_check_memory_is_clean)
        ;; store the function result
        (local.set $clean)

        ;; Encode () in SBOR at address 0x0
        (i32.const 0)
        (i32.const 92)  ;; prefix
        (i32.store8)

        (i32.const 1)
        (i32.const 1)  ;; tuple value kind bool
        (i32.store8)

        (i32.const 2)
        (local.get $clean)
        (i64.store8)   ;; wrap clean to i8 and store

        ;; Return slice (ptr = 0, len = 3)
        (i64.const 3)
    )
    (memory $0 1)
    (export "memory" (memory $0))
    (export "Test_grow_memory" (func $Test_grow_memory))
    (export "Test_read_memory" (func $Test_read_memory))
    (export "Test_write_memory" (func $Test_write_memory))
    (export "Test_check_memory_is_clean" (func $Test_check_memory_is_clean))
)
