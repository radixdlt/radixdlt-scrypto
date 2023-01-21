(module

  ;; Simple function that always returns `()`
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

  (memory $0 100)
  (export "memory" (memory $0))
  (export "Test_f" (func $Test_f))
)