;; LLVM memmove builtin
(func $memmove (param i32 i32 i32) (result i32)
  (local i32 i32 i32 i32 i32 i32 i32 i32)
  block
    block
      block
        block
          local.get 0
          local.get 1
          i32.sub
          local.get 2
          i32.ge_u
          br_if 0
          local.get 1
          local.get 2
          i32.add
          local.set 3
          local.get 0
          local.get 2
          i32.add
          local.set 4
          block
            local.get 2
            i32.const 15
            i32.gt_u
            br_if 0
            local.get 0
            local.set 5
            br 3
          end
          local.get 4
          i32.const -4
          i32.and
          local.set 6
          i32.const 0
          local.get 4
          i32.const 3
          i32.and
          local.tee 7
          i32.sub
          local.set 8
          block
            local.get 7
            i32.eqz
            br_if 0
            local.get 1
            local.get 2
            i32.add
            i32.const -1
            i32.add
            local.set 5
            loop
              local.get 4
              i32.const -1
              i32.add
              local.tee 4
              local.get 5
              i32.load8_u
              i32.store8
              local.get 5
              i32.const -1
              i32.add
              local.set 5
              local.get 6
              local.get 4
              i32.lt_u
              br_if 0
            end
          end
          local.get 6
          local.get 2
          local.get 7
          i32.sub
          local.tee 9
          i32.const -4
          i32.and
          local.tee 5
          i32.sub
          local.set 4
          i32.const 0
          local.get 5
          i32.sub
          local.set 7
          block
            local.get 3
            local.get 8
            i32.add
            local.tee 8
            i32.const 3
            i32.and
            i32.eqz
            br_if 0
            local.get 7
            i32.const -1
            i32.gt_s
            br_if 2
            local.get 8
            i32.const 3
            i32.shl
            local.tee 5
            i32.const 24
            i32.and
            local.set 2
            local.get 8
            i32.const -4
            i32.and
            local.tee 10
            i32.const -4
            i32.add
            local.set 1
            i32.const 0
            local.get 5
            i32.sub
            i32.const 24
            i32.and
            local.set 3
            local.get 10
            i32.load
            local.set 5
            loop
              local.get 6
              i32.const -4
              i32.add
              local.tee 6
              local.get 5
              local.get 3
              i32.shl
              local.get 1
              i32.load
              local.tee 5
              local.get 2
              i32.shr_u
              i32.or
              i32.store
              local.get 1
              i32.const -4
              i32.add
              local.set 1
              local.get 6
              local.get 4
              i32.gt_u
              br_if 0
              br 3
            end
          end
          local.get 7
          i32.const -1
          i32.gt_s
          br_if 1
          local.get 9
          local.get 1
          i32.add
          i32.const -4
          i32.add
          local.set 1
          loop
            local.get 6
            i32.const -4
            i32.add
            local.tee 6
            local.get 1
            i32.load
            i32.store
            local.get 1
            i32.const -4
            i32.add
            local.set 1
            local.get 6
            local.get 4
            i32.gt_u
            br_if 0
            br 2
          end
        end
        block
          block
            local.get 2
            i32.const 15
            i32.gt_u
            br_if 0
            local.get 0
            local.set 4
            br 1
          end
          local.get 0
          i32.const 0
          local.get 0
          i32.sub
          i32.const 3
          i32.and
          local.tee 3
          i32.add
          local.set 5
          block
            local.get 3
            i32.eqz
            br_if 0
            local.get 0
            local.set 4
            local.get 1
            local.set 6
            loop
              local.get 4
              local.get 6
              i32.load8_u
              i32.store8
              local.get 6
              i32.const 1
              i32.add
              local.set 6
              local.get 4
              i32.const 1
              i32.add
              local.tee 4
              local.get 5
              i32.lt_u
              br_if 0
            end
          end
          local.get 5
          local.get 2
          local.get 3
          i32.sub
          local.tee 8
          i32.const -4
          i32.and
          local.tee 9
          i32.add
          local.set 4
          block
            block
              local.get 1
              local.get 3
              i32.add
              local.tee 7
              i32.const 3
              i32.and
              i32.eqz
              br_if 0
              local.get 9
              i32.const 1
              i32.lt_s
              br_if 1
              local.get 7
              i32.const 3
              i32.shl
              local.tee 6
              i32.const 24
              i32.and
              local.set 2
              local.get 7
              i32.const -4
              i32.and
              local.tee 10
              i32.const 4
              i32.add
              local.set 1
              i32.const 0
              local.get 6
              i32.sub
              i32.const 24
              i32.and
              local.set 3
              local.get 10
              i32.load
              local.set 6
              loop
                local.get 5
                local.get 6
                local.get 2
                i32.shr_u
                local.get 1
                i32.load
                local.tee 6
                local.get 3
                i32.shl
                i32.or
                i32.store
                local.get 1
                i32.const 4
                i32.add
                local.set 1
                local.get 5
                i32.const 4
                i32.add
                local.tee 5
                local.get 4
                i32.lt_u
                br_if 0
                br 2
              end
            end
            local.get 9
            i32.const 1
            i32.lt_s
            br_if 0
            local.get 7
            local.set 1
            loop
              local.get 5
              local.get 1
              i32.load
              i32.store
              local.get 1
              i32.const 4
              i32.add
              local.set 1
              local.get 5
              i32.const 4
              i32.add
              local.tee 5
              local.get 4
              i32.lt_u
              br_if 0
            end
          end
          local.get 8
          i32.const 3
          i32.and
          local.set 2
          local.get 7
          local.get 9
          i32.add
          local.set 1
        end
        local.get 2
        i32.eqz
        br_if 2
        local.get 4
        local.get 2
        i32.add
        local.set 5
        loop
          local.get 4
          local.get 1
          i32.load8_u
          i32.store8
          local.get 1
          i32.const 1
          i32.add
          local.set 1
          local.get 4
          i32.const 1
          i32.add
          local.tee 4
          local.get 5
          i32.lt_u
          br_if 0
          br 3
        end
      end
      local.get 9
      i32.const 3
      i32.and
      local.tee 1
      i32.eqz
      br_if 1
      local.get 8
      local.get 7
      i32.add
      local.set 3
      local.get 4
      local.get 1
      i32.sub
      local.set 5
    end
    local.get 3
    i32.const -1
    i32.add
    local.set 1
    loop
      local.get 4
      i32.const -1
      i32.add
      local.tee 4
      local.get 1
      i32.load8_u
      i32.store8
      local.get 1
      i32.const -1
      i32.add
      local.set 1
      local.get 5
      local.get 4
      i32.lt_u
      br_if 0
    end
  end
  local.get 0)
