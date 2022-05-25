(module
  (memory $0 1)

  ;; Store the ABI at global address 1024
  (data (i32.const 1024) "#\03\00\00\00\11\06\00\00\00Struct\02\00\00\00\0c\04\00\00\00Test\11\04\00\00\00Unit\00\00\00\000\10\00\00\00\000\10\00\00\00\00")

  ;; Static Scrypto buffer allocation
  (func $scrypto_alloc (param $0 i32) (result i32)
    (i32.store8 offset=0
      (i32.const 0)
      (local.get $0)
    )
    (i32.store8 offset=1
      (i32.const 0)
      (i32.shr_u
        (local.get $0)
        (i32.const 8)
      )
    )
    (i32.store8 offset=2
      (i32.const 0)
      (i32.shr_u
        (local.get $0)
        (i32.const 16)
      )
    )
    (i32.store8 offset=3
      (i32.const 0)
      (i32.shr_u
        (local.get $0)
        (i32.const 24)
      )
    )
    (i32.const 0)
  )

  ;; Static Scrypto buffer release
  (func $scrypto_free (param $0 i32)
  )

  ;; Simple main function that always returns `()`
  (func $Test_main (param $0 i32) (result i32)
    ;; Start!
    ;; ```
    ;; int sum = 0;
    ;; for (int i = 0; i < 100; i++) {
    ;;   sum += i;
    ;; }
    ;; ```
    (local $1 i32)
    (i32.store offset=12
      (local.tee $1
        (i32.sub
          (i32.load offset=4
            (i32.const 0)
          )
          (i32.const 16)
        )
      )
      (i32.const 0)
    )
    (i32.store offset=8
      (local.get $1)
      (i32.const 0)
    )
    (i32.store offset=4
      (local.get $1)
      (i32.const 0)
    )
    (block $label$0
      (loop $label$1
        (br_if $label$0
          (i32.gt_s
            (i32.load offset=4
              (local.get $1)
            )
            (i32.const ${n_minus_one})
          )
        )
        (i32.store offset=8
          (local.get $1)
          (i32.add
            (i32.load offset=8
              (local.get $1)
            )
            (i32.load offset=4
              (local.get $1)
            )
          )
        )
        (i32.store offset=4
          (local.get $1)
          (i32.add
            (i32.load offset=4
              (local.get $1)
            )
            (i32.const 1)
          )
        )
        (br $label$1)
      )
    )
    ;; End!

    (i32.add
      (call $scrypto_alloc
        (i32.const 1)
      )
      (i32.const 4)
    )
    (i32.const 0)
    (i32.store8)
    (i32.const 0)
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
  
  (export "memory" (memory $0))
  (export "scrypto_alloc" (func $scrypto_alloc))
  (export "scrypto_free" (func $scrypto_free))
  (export "Test_main" (func $Test_main))
  (export "Test_abi" (func $Test_abi))

  ${builtin_memcpy}
)