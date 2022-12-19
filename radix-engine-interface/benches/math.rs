use criterion::{criterion_group, criterion_main};
mod macros;
mod decimal;
mod integer;

use decimal::*;
use integer::*;

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

    bench_i256_add,
    bench_i256_sub,
    bench_i256_mul,
    bench_i256_div,
    bench_i256_root,
    bench_i256_pow,
    bench_i256_from_string,
    bench_i256_to_string,

    bench_i512_add,
    bench_i512_sub,
    bench_i512_mul,
    bench_i512_div,
    bench_i512_root,
    bench_i512_pow,
    bench_i512_from_string,
    bench_i512_to_string
);
criterion_main!(bench_math);
