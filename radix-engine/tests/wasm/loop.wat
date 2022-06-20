(module

  ;; Simple function that always returns `()`
  (func $Test_f (param $0 i32) (result i32)
    ;; Loop starts!
    (local $i i32)
    (loop $loop

      ;; Add one to $i
      local.get $i
      i32.const 1
      i32.add
      local.set $i

      ;; If $i < 10, branch to loop
      local.get $i
      i32.const ${n}
      i32.lt_s
      br_if $loop
    )
    ;; Loop ends!

    (local.set 
      $0
      (call $scrypto_alloc
        (i32.const 1)
      )
    )
    (i32.add
      (local.get $0)
      (i32.const 4)
    )
    (i32.const 0)
    (i32.store8)
    (local.get $0)
  )

  (memory $0 1)
  (export "memory" (memory $0))
  (export "scrypto_alloc" (func $scrypto_alloc))
  (export "scrypto_free" (func $scrypto_free))
  (export "Test_f" (func $Test_f))

  ${memcpy}
  ${buffer}
)