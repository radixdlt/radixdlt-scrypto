use criterion::{criterion_group, criterion_main};
mod macros;
mod decimal;
mod precise_decimal;
mod integer;

use decimal::*;
use precise_decimal::*;
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

    bench_precisedecimal_add,
    bench_precisedecimal_sub,
    bench_precisedecimal_mul,
    bench_precisedecimal_div,
    bench_precisedecimal_root,
    bench_precisedecimal_pow,
    bench_precisedecimal_from_string,
    bench_precisedecimal_to_string,

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
    bench_i512_to_string,

    bench_bigint_add,
    bench_bigint_sub,
    bench_bigint_mul,
    bench_bigint_div,
    bench_bigint_root,
    bench_bigint_pow,
    bench_bigint_from_string,
    bench_bigint_to_string,

    bench_integer_add,
    bench_integer_sub,
    bench_integer_mul,
    bench_integer_div,
    bench_integer_root,
    bench_integer_pow,
    bench_integer_from_string,
    bench_integer_to_string,

    bench_bnumbint256_add,
    bench_bnumbint256_sub,
    bench_bnumbint256_mul,
    bench_bnumbint256_div,
    bench_bnumbint256_root,
    bench_bnumbint256_pow,
    bench_bnumbint256_from_string,
    bench_bnumbint256_to_string,

    bench_ethnumi256_add,
    bench_ethnumi256_sub,
    bench_ethnumi256_mul,
    bench_ethnumi256_div,
//    bench_ethnumi256_root,
    bench_ethnumi256_pow,
    bench_ethnumi256_from_string,
    bench_ethnumi256_to_string,
);
criterion_main!(bench_math);
