use criterion::{criterion_group, criterion_main};
mod macros;
mod decimal;

use decimal::*;

criterion_group!(
    bench_math,
    bench_decimal_add,
    bench_decimal_sub,
    bench_decimal_mul,
    bench_decimal_div,
    bench_decimal_root,
    bench_decimal_pow,
    bench_decimal_from_string,
    bench_decimal_to_string,
);
criterion_main!(bench_math);
