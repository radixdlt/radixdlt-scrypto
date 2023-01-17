(module

  ;; Simple function that always returns `()`
  ;; Need to replace ${depth} with the depth
  (func $Test_f (param $0 i64) (result i64)
    ;; Loop starts!
    (local $i i32)
    (local $payload i32)
    (local $payload_length i32)
    (local $curr_pointer i32)

    ;; Get the pointer to paylod
    (local.set 
      $payload
      (i32.const 0)
    )

    ;; Paylod length = 2 * depth + 1
    (local.set
      $payload_length
      (i32.add
        (i32.mul
            (i32.const 2)
            (i32.const ${depth})
        )
        (i32.const 1)
      )
    )

    ;; Set up current pointer
    (local.set
      $curr_pointer
      (i32.const 0)
    )

    ;; Write SBOR prefix
    local.get $curr_pointer
    i32.const 92
    i32.store8

    (local.set
      $curr_pointer
      (i32.add
        (local.get $payload)
        (i32.const 1)
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

      ;; Write Tuple value kind, and push the pointer forward one
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
    
    ;; Return slice (ptr = 0, len)
    (i64.extend_i32_s (local.get $payload_length))
  )

  (memory $0 1)
  (export "memory" (memory $0))
  (export "Test_f" (func $Test_f))
)