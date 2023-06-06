(module
    ;; Template file to test WASM sign-extension ops.
    ;; Function extends into a given type (i32 or i64) given number of bits from the
    ;; initial value of the same type (i32 or i64) as and returns the value as SBOR
    ;; value of the respective value kind.

    ;; Replace before compiling:
    ;;   ${value_kind} with i32 or i64 value kind codes
    ;;   ${initial} with value to convert from
    ;;   ${base} with i32 or i64 type
    ;;   ${instruction} with extend instruction: extend8_s, extend16_s or extend32_s
    ;;   ${slice_len} length of the slice return (depends on value kind, i32 vs i64)
    (func $Test_f (param $0 i64) (result i64)

        ;; Encode () in SBOR at address 0x0
        (i32.const 0)
        (i32.const 92)  ;; prefix
        (i32.store8)

        (i32.const 1)
        (i32.const ${value_kind})  ;; i32 value kind
        (i32.store8)


        (i32.const 2)
        (${base}.const ${initial})     ;; Push initial value onto the stack
        (${base}.${instruction})       ;; Take given number of bits (8, 16 or 32) from the initial value
                                       ;; and extend them into a the same type as initial value
                                       ;; and put it back onto the stack.
        (${base}.store)                ;; Store all of the bytes into memory

        ;; Return slice (ptr = 0, len = 6 (1 prefix + 1 value kind + 4 i32 len))
        (i64.const ${slice_len})
    )
    (memory $0 1)
    (export "memory" (memory $0))
    (export "Test_f" (func $Test_f))
)

