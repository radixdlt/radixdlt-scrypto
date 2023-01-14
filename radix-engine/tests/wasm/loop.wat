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

      ;; If $i < ${n}, branch to loop
      local.get $i
      i32.const ${n}
      i32.lt_s
      br_if $loop
    )
    ;; Loop ends!

    ;; TO RETURN:
    ;; Now we need to allocate the return SBOR buffer: We need 3 bytes to respond with ()
    ;; $scrypto_alloc returns a pointer - the first 4 bytes aren't relevant to use, we start writing our response after that
    (local.set 
      $0
      (call $scrypto_alloc
        (i32.const 3)
      )
    )

    ;; PART 1: Encode our Scrypto payload prefix (92) as a byte (8 bits), at offset 4 from the pointer
    (i32.add
      (local.get $0)
      (i32.const 4)
    )
    (i32.const 92)
    (i32.store8)

    ;; PART 2: We need to write two little endian bytes of 0x2100 to encode (), at offset 4 + 1 from the pointer
    (i32.add
      (local.get $0)
      (i32.const 5)
    )
    (i32.const 0x0021)
    (i32.store16)

    ;; We're finished! Return the pointer
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