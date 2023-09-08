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

        ;; Return slice of len = 0
        (i64.const 0)
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

        ;; Return slice of len = 0
        (i64.const 0)
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

        ;; Return slice of len = 0
        (i64.const 0)
    )

    (func $Test_check_memory_is_clean (result i64)
        (local $clean i64)

        (call $host_check_memory_is_clean)
        ;; store the function result
        (local.set $clean)

        ;; store clean flag in the first byte
        (i32.const 0)
        (local.get $clean)
        (i64.store8)   ;; wrap clean to i8 and store

        ;; Return slice (ptr = 0, len = 1)
        (i64.const 1)
    )
    (memory $0 1)
    (export "memory" (memory $0))
    (export "Test_grow_memory" (func $Test_grow_memory))
    (export "Test_read_memory" (func $Test_read_memory))
    (export "Test_write_memory" (func $Test_write_memory))
    (export "Test_check_memory_is_clean" (func $Test_check_memory_is_clean))
)
