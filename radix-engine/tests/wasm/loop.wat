(module
  ;; Store the ABI at global address 1024
  (data (i32.const 1024) "#\03\00\00\00\11\06\00\00\00Struct\02\00\00\00\0c\04\00\00\00Test\11\04\00\00\00Unit\00\00\00\000\10\00\00\00\000\10\00\00\00\00")

  ;; Simple main function that always returns `()`
  (func $Test_main (param $0 i32) (param $1 i32) (result i32)
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

  ;; Simple ABI of unit blueprint with no function or method
  (func $Test_abi (param $0 i32) (param $1 i32) (result i32)
    (local.set 
      $0
      (call $scrypto_alloc
        (i32.const 54)
      )
    )
    (drop
      (call $memcpy
        (i32.add
          (local.get $0)
          (i32.const 4)
        )
        (i32.const 1024)
        (i32.const 54)
      )
    )
    (local.get $0)
  )
  
  (memory $0 1)
  (export "memory" (memory $0))
  (export "scrypto_alloc" (func $scrypto_alloc))
  (export "scrypto_free" (func $scrypto_free))
  (export "Test_main" (func $Test_main))
  (export "Test_abi" (func $Test_abi))

  ${memcpy}
  ${buffer}
)