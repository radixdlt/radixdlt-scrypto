use criterion::{criterion_group, criterion_main};
mod decimal;
mod integer;
mod macros;
mod precise_decimal;

use decimal::*;
use integer::*;
use precise_decimal::*;

criterion_group!(
    bench_decimal,
    bench_decimal_add,
    bench_decimal_sub,
    bench_decimal_mul,
    bench_decimal_div,
    bench_decimal_root,
    bench_decimal_pow,
    bench_decimal_from_string,
    bench_decimal_to_string,
);
criterion_group!(
    bench_precise_decimal,
    bench_precisedecimal_add,
    bench_precisedecimal_sub,
    bench_precisedecimal_mul,
    bench_precisedecimal_div,
    bench_precisedecimal_root,
    bench_precisedecimal_pow,
    bench_precisedecimal_from_string,
    bench_precisedecimal_to_string,
);
criterion_group!(
    bench_bigint,
    bench_bigint_add,
    bench_bigint_sub,
    bench_bigint_mul,
    bench_bigint_div,
    bench_bigint_root,
    bench_bigint_pow,
    bench_bigint_from_string,
    bench_bigint_to_string,
);
criterion_group!(
    bench_rug,
    bench_integer_add,
    bench_integer_sub,
    bench_integer_mul,
    bench_integer_div,
    bench_integer_root,
    bench_integer_pow,
    bench_integer_from_string,
    bench_integer_to_string,
);
criterion_group!(
    bench_bnumbint,
    bench_bnumbint256_add,
    bench_bnumbint256_sub,
    bench_bnumbint256_mul,
    bench_bnumbint256_div,
    bench_bnumbint256_root,
    bench_bnumbint256_pow,
    bench_bnumbint256_from_string,
    bench_bnumbint256_to_string,
);
criterion_group!(
    bench_ethnumi256,
    bench_ethnumi256_add,
    bench_ethnumi256_sub,
    bench_ethnumi256_mul,
    bench_ethnumi256_div,
    // bench_ethnumi256_root,
    bench_ethnumi256_pow,
    bench_ethnumi256_from_string,
    bench_ethnumi256_to_string,
);
criterion_main!(bench_decimal, bench_precise_decimal);

// criterion_main!(
//     bench_decimal,
//     bench_precise_decimal,
//     bench_i256,
//     bench_i512,
//     bench_bigint,
//     bench_rug,
//     bench_bnumbint,
//     bench_ethnumi256
// );
