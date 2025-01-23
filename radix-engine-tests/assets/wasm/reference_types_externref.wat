(module
  ;; Define a table with a externref items
  (table $ref_table 2 externref)

  ;; Exported function that adds two values returned by $return_two_values
  (func $Test_f (param $0 i64) (result i64)

    ;; Encode () in SBOR at address 0x0
    (i32.const 0)
    (i32.const 92)  ;; prefix
    (i32.store8)
    (i32.const 1)
    (i32.const 4)  ;; i32 value kind
    (i32.store8)
    (i32.const 2)
    (i32.const 0)  ;; value
    (i32.store)

    ;; Return slice (ptr = 0, len = 6)
    (i64.const 6)
  )

  ;; Define memory with an initial size of 1 page (64 KiB)
  (memory $0 1)
  (export "memory" (memory $0))
  (export "Test_f" (func $Test_f))
)

