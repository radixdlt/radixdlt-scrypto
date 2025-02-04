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

  ;; Function that overwrites entry at given index in function table
  (func $Test_f (param $0 i64) (result i64)
    (local $result i32)

    ;; Overwrite function at given index
    (table.set $t
      (i32.const ${index})    ;; Index
      (ref.func $add)         ;; Reference to $add function
    )

    ;; Encode () in SBOR at address 0x0
    (i32.const 0)
    (i32.const 92)  ;; prefix
    (i32.store8)
    (i32.const 1)
    (i32.const 4)  ;; i32 value kind
    (i32.store8)
    (i32.const 2)
    (i32.const 0)  ;; value
    (i32.store)

    ;; Return slice (ptr = 0, len = 6)
    (i64.const 6)
  )

  ;; Define memory with an initial size of 1 page (64 KiB)
  (memory $0 1)
  (export "memory" (memory $0))
  (export "Test_f" (func $Test_f))
)
