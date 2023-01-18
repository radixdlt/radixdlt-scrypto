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
      (i32.add
        ;; The first byte of the return is for the prefix byte 0x92
        (i32.const 1)
        ;; We are going to encode two bytes (TUPLE_VALUE_KIND and TUPLE_LENGTH), depth times, to create a tuple "depth" deep.
        ;; The first depth - 1 tuples will have a length of 1, the last will have a length of 0
        (i32.mul
          (i32.const 2)
          (i32.const ${depth})
        )
      )
    )

    ;; Get the pointer to write the response at, and store to $0 - this is the pointer we'll return at the end of the method
    (local.set $0 (call $scrypto_alloc (local.get $return_length)))

    ;; Set the $curr_pointer to where we will write out response.
    ;; We will skip the first 4 bytes from the offset, and commence writing our response there
    (local.set $curr_pointer (i32.add (local.get $0) (i32.const 4)))

    ;; PART 1: Encode our Scrypto payload prefix (0x5c) as a byte (8 bits), and advance the $curr_pointer by 1 byte
    (i32.store8 (local.get $curr_pointer) (i32.const 0x5c))
    (local.set $curr_pointer (i32.add (local.get $curr_pointer) (i32.const 1)))

    ;; Set i = 1 - it'll get incremented to 2 before the first body of the loop
    (local.set $i (i32.const 1))
  
    ;; Now start our loop
    ;; The body of the loop runs with i = 2 up to i = depth inclusive...
    ;; So depth - 1 times in total -- and then we'll add the last depth with the unit at the end!
    (loop $loop
      ;; Add one to $i
      (local.set $i (i32.add (local.get $i) (i32.const 1)))

      ;; Write Tuple Type, and push the pointer forward one
      (i32.store8 (local.get $curr_pointer) (i32.const 33))
      (local.set $curr_pointer (i32.add (local.get $curr_pointer) (i32.const 1)))

      ;; Write Tuple length of 1, and push the pointer forward one
      (i32.store8 (local.get $curr_pointer) (i32.const 1))
      (local.set $curr_pointer (i32.add (local.get $curr_pointer) (i32.const 1)))

      ;; If $i < ${depth}, branch to loop
      (i32.lt_s (local.get $i) (i32.const ${depth}))
      br_if $loop
    )

    ;; Write two little endian bytes of 0x2100 to encode () and finish off
    (i32.store16 (local.get $curr_pointer) (i32.const 0x0021))
  
    ;; Return the buffer start in variable $0
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