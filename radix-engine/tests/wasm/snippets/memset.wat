;; LLVM memset builtin
(func $memset (param i32 i32 i32) (result i32)
  (local i32 i32 i32)
  block
    block
      local.get 2
      i32.const 15
      i32.gt_u
      br_if 0
      local.get 0
      local.set 3
      br 1
    end
    local.get 0
    i32.const 0
    local.get 0
    i32.sub
    i32.const 3
    i32.and
    local.tee 4
    i32.add
    local.set 5
    block
      local.get 4
      i32.eqz
      br_if 0
      local.get 0
      local.set 3
      loop
        local.get 3
        local.get 1
        i32.store8
        local.get 3
        i32.const 1
        i32.add
        local.tee 3
        local.get 5
        i32.lt_u
        br_if 0
      end
    end
    local.get 5
    local.get 2
    local.get 4
    i32.sub
    local.tee 4
    i32.const -4
    i32.and
    local.tee 2
    i32.add
    local.set 3
    block
      local.get 2
      i32.const 1
      i32.lt_s
      br_if 0
      local.get 1
      i32.const 255
      i32.and
      i32.const 16843009
      i32.mul
      local.set 2
      loop
        local.get 5
        local.get 2
        i32.store
        local.get 5
        i32.const 4
        i32.add
        local.tee 5
        local.get 3
        i32.lt_u
        br_if 0
      end
    end
    local.get 4
    i32.const 3
    i32.and
    local.set 2
  end
  block
    local.get 2
    i32.eqz
    br_if 0
    local.get 3
    local.get 2
    i32.add
    local.set 5
    loop
      local.get 3
      local.get 1
      i32.store8
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get 5
      i32.lt_u
      br_if 0
    end
  end
  local.get 0)
