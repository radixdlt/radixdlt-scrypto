(module
  ;; Some recursive function
  (func $f (param $0 i32) (result i32)
    (if
      (i32.lt_s
        (local.get $0)
        (i32.const 2)
      )
      (return
        (i32.const 1)
      )
    )
    (return
      (i32.add
        (call $f
          (i32.sub
            (local.get $0)
            (i32.const 1)
          )
        )
        (local.get $0)
      )
    )
  )

  ;; Simple function that always returns `()`
  (func $Test_f (param $0 i64) (result i64)
    ;; Recursion starts!
    (drop
      (call $f
        (i32.sub
          (i32.const ${n})
          (i32.const 1)
        )
      )
    )
    ;; Recursion ends!

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