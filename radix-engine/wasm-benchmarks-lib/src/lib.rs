mod decimal;
mod precise_decimal;
mod primitive;

pub use decimal::*;
pub use precise_decimal::*;
pub use primitive::*;

// This is used to call from WASM any native method from this crate, provided that the method was imported before
pub const WAT_CALL_HOST: &str = r#"
        (module
            (import "host" "host_func" (func $host_func (param i64 i64 i64) (result i64)))
            (func $local_func (param i64 i64 i64) (result i64)
                local.get 0
                local.get 1
                local.get 2
                call $host_func
            )
            (export "local_call_host_func" (func $local_func))
        )
    "#;
