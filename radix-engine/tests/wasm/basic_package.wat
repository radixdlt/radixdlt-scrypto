(module
  ;; Store the ABI at global address 1024
  (data (i32.const 1024) "#\03\00\00\00\11\06\00\00\00Struct\02\00\00\00\0c\04\00\00\00Test\11\04\00\00\00Unit\00\00\00\000\10\00\00\00\000\10\00\00\00\00")

  ;; Simple main function that always returns `()`
  (func $Test_main (param $0 i32) (result i32)
    (local $buffer i32)
    (local.set 
      $buffer
      (call $scrypto_alloc
        (i32.const 1)
      )
    )
    (i32.add
      (local.get $buffer)
      (i32.const 4)
    )
    (i32.const 0)
    (i32.store8)
    (local.get $buffer)
  )

  ;; Simple ABI of unit blueprint with no function or method
  (func $Test_abi (param $0 i32) (result i32)
    (local $buffer i32)
    (local.set 
      $buffer
      (call $scrypto_alloc
        (i32.const 54)
      )
    )
    (drop
      (call $memcpy
        (i32.add
          (local.get $buffer)
          (i32.const 4)
        )
        (i32.const 1024)
        (i32.const 54)
      )
    )
    (local.get $buffer)
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