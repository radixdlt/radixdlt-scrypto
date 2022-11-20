(module

  ;; Simple function that always returns `()`
  ;; Need to replace ${n} with the depth
  (func $Test_f (param $0 i32) (result i32)
    ;; Loop starts!
    (local $i i32)
    (local $curr_pointer i32)
    (local $return_length i32)

    ;; Return length needs 2 + 2 * n
    ;; It needs 4 initial bytes, plus (n-2) * 2 bytes in the middle, plus 2 bytes for the end
    (local.set
      $return_length
      (i32.add
        (i32.const 2)
        (i32.mul
          (i32.const 2)
          (i32.const ${n})
        )
      )
    )

    ;; Return stuff
    (local.set 
      $0
      (call $scrypto_alloc
        (local.get $return_length)
      )
    )

    (local.set
      $curr_pointer
      (i32.add
        (local.get $0)
        (i32.const 4)
      )
    )
  
    ;; Now start our loop
    (loop $loop
      ;; Add one to $i
      local.get $i
      i32.const 1
      i32.add
      local.set $i

      ;; Write Tuple Type, and push the pointer forward one
      local.get $curr_pointer
      i32.const 33
      i32.store8

      (local.set
        $curr_pointer
        (i32.add
          (local.get $curr_pointer)
          (i32.const 1)
        )
      )

      ;; Write Tuple length of 1, and push the pointer forward one
      local.get $curr_pointer
      i32.const 1
      i32.store8

      (local.set
        $curr_pointer
        (i32.add
          (local.get $curr_pointer)
          (i32.const 1)
        )
      )

      ;; If $i < ${n}, branch to loop
      local.get $i
      i32.const ${n}
      i32.lt_s
      br_if $loop
    )

    ;; Write a unit 0x0000 to finish off
    local.get $curr_pointer
    (i32.const 0)
    (i32.store16)
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