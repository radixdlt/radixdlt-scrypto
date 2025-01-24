(module
  ;; Define a function table with an initial size of 3 and a maximum size of 4
  (table $t 3 4 funcref)
  (type $func_type (func (param i32 i32) (result i32)))

  ;; Initialize the table with the 3 functions
  (elem (i32.const 0) $add $multiply $subtract)

  ;; Function 0: Adds two numbers
  (func $add (type $func_type)
    (local.get 0)
    (local.get 1)
    (i32.add)
  )

  ;; Function 1: Multiplies two numbers
  (func $multiply (type $func_type)
    (local.get 0)
    (local.get 1)
    (i32.mul)
  )

  ;; Function 2: Subtracts two numbers
  (func $subtract (type $func_type)
    (local.get 0)
    (local.get 1)
    (i32.sub)
  )

  ;; Function to grow the table and add a new element
  (func $grow_table
    (local $ref_func funcref)
    ;; ref.func not yet supported in 'wasm-instrument'
    ;; see https://github.com/radixdlt/wasm-instrument/blob/405166c526aa60fa2af4e4b1122b156dbcc1bb15/src/stack_limiter/max_height.rs#L455
    ;; (local.set $ref_func
    ;;   (ref.func $add)
    ;; )
    ;; use table.get instead to get funcref
    (local.set $ref_func
      (table.get $t
        ;; Get $add function at the index 0
        (i32.const 0)
      )
    )
    (table.grow $t
      (local.get $ref_func) ;; Initial value of the new entries
      (i32.const 1)         ;; Number of entries to grow
    )
    ;; table.grow returns previous size, drop it
    (drop)
  )

  ;; Function that grows table and calls some function in the table
  (func $Test_f (param $0 i64) (result i64)
    (local $result i32)

    (call $grow_table)

    (i32.const ${a})
    (i32.const ${b})
    (call_indirect
      (type $func_type)
      (i32.const ${index}) ;; Call function at given index
    )
    (local.set $result)  ;; value

    ;; Encode () in SBOR at address 0x0
    (i32.const 0)
    (i32.const 92)  ;; prefix
    (i32.store8)
    (i32.const 1)
    (i32.const 4)  ;; i32 value kind
    (i32.store8)
    (i32.const 2)
    (local.get $result)  ;; value
    (i32.store)

    ;; Return slice (ptr = 0, len = 6)
    (i64.const 6)
  )

  ;; Define memory with an initial size of 1 page (64 KiB)
  (memory $0 1)
  (export "memory" (memory $0))
  (export "Test_f" (func $Test_f))
)
