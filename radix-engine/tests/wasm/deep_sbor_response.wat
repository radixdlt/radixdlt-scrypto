(module

  ;; Simple function that always returns `()`
  ;; Need to replace ${depth} with the depth
  (func $Test_f (param $0 i32) (result i32)
    ;; Loop starts!
    (local $i i32)
    (local $curr_pointer i32)
    (local $return_length i32)

    ;; Return length needs 2 * depth
    ;; It needs (depth-1) * 2 bytes in the middle, plus 2 bytes for the end
    (local.set
      $return_length
      (i32.mul
        (i32.const 2)
        (i32.const ${depth})
      )
    )

    ;; Get the pointer to write the response at
    (local.set 
      $0
      (call $scrypto_alloc
        (i32.add
          (local.get $return_length)
          (i32.const 1)
        )
      )
    )

    ;; But skip the first 5 padder bytes before writing the response!
    (local.set
      $curr_pointer
      (i32.add
        (local.get $0)
        (i32.const 5)
      )
    )

    ;; Set i = 1 - it'll get incremented to 2 before the first body of the loop
    i32.const 1
    local.set $i
  
    ;; Now start our loop
    ;; The body of the loop runs with i = 2 up to i = depth inclusive...
    ;; So depth - 1 times in total -- and then we'll add the last depth with the unit at the end!
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

      ;; If $i < ${depth}, branch to loop
      local.get $i
      i32.const ${depth}
      i32.lt_s
      br_if $loop
    )

    ;; Write two little endian bytes of 0x2100 to encode () and finish off
    local.get $curr_pointer
    i32.const 0x0021
    i32.store16
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