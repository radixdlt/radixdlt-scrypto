;; Simple scrypto buffer allocator that uses static addresses
;; Note that a WebAssembly page has a constant size of 64KiB
(func $scrypto_alloc (param $0 i32) (result i32)
  (i32.store8 offset=4096
    (i32.const 0)
    (local.get $0)
  )
  (i32.store8 offset=4097
    (i32.const 0)
    (i32.shr_u
      (local.get $0)
      (i32.const 8)
    )
  )
  (i32.store8 offset=4098
    (i32.const 0)
    (i32.shr_u
      (local.get $0)
      (i32.const 16)
    )
  )
  (i32.store8 offset=4099
    (i32.const 0)
    (i32.shr_u
      (local.get $0)
      (i32.const 24)
    )
  )
  (i32.const 4096)
)

(func $scrypto_free (param $0 i32)
)