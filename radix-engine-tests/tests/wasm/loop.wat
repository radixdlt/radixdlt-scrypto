(module

  ;; Simple function that always returns `()`
  (func $Test_f (param $0 i64) (result i64)
    ;; Loop starts!
    (local $i i32)
    (loop $loop

      ;; Add one to $i
      local.get $i
      i32.const 1
      i32.add
      local.set $i

      ;; If $i < ${n}, branch to loop
      local.get $i
      i32.const ${n}
      i32.lt_s
      br_if $loop
    )
    ;; Loop ends!

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