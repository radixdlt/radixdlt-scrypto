use std::concat;

use criterion::{BenchmarkId, Criterion};
use radix_engine_interface::math::Decimal;

use crate::{bench_ops,process_op};


fn decimal_add(a: Decimal, b: Decimal   ) {
    let _ = a + b;
}

fn decimal_sub(a: Decimal, b: Decimal) {
    let _ = a - b;
}
fn decimal_mul(a: Decimal, b: Decimal) {
    let _ = a * b;
}

fn decimal_div(a: Decimal, b: Decimal) {
    let _ = a / b;
}

fn decimal_root(a: Decimal, n: u32) {
    let _ = a.nth_root(n);
}

fn decimal_pow(a: Decimal, exp: i64) {
    let _ = a.powi(exp);
}

fn decimal_to_string(a: Decimal, _: &str) {
    let _ = a.to_string();
}

fn decimal_from_string(s: &str, _: &str) {
    let _ = Decimal::from(s);
}

//#![feature(trace_macros)]
//trace_macros!(true);

const ADD_OPERANDS: [(&str, &str); 4] = [
    ("27896044618658097711785492504343953926634900000000000000000.12312312312312", "27896044618658097711785492504343953926634900000000000000000.12312312312312"),
    ("-27896044618658097711785492504343953926634900000000000000000.12312312312312", "27896044618658097711785492504343953926634900000000000000000.12312312312312"),
    ("1", "-1"),
    ("-27896044618658097711785492504343953926634900000000000000000.12312312312312", "-27896044618658097711785492504343953926634900000000000000000.12312312312312"),
];

const SUB_OPERANDS: [(&str, &str); 4] = [
    ("27896044618658097711785492504343953926634900000000000000000.12312312312312", "27896044618658097711785492504343953926634900000000000000000.12312312312312"),
    ("-27896044618658097711785492504343953926634900000000000000000.12312312312312", "27896044618658097711785492504343953926634900000000000000000.12312312312312"),
    ("1", "-1"),
    ("-27896044618658097711785492504343953926634900000000000000000.12312312312312", "-27896044618658097711785492504343953926634900000000000000000.12312312312312"),
];

const MUL_OPERANDS: [(&str, &str); 4] = [
    ("278960446186580977117.854925043439539", "278960446186580977117.8549250434395392"),
    ("-278960446186580977117.854925043439539", "278960446186580977117.8549250434395392"),
    ("63499233282.0282019728", "1312.31233"),
    ("-123123123123", "-1"),
];

const DIV_OPERANDS: [(&str, &str); 4] = [
    ("278960446186580977117.854925043439539", "278960446186580977117.8549250434395392"),
    ("-278960446186580977117.854925043439539", "278960446186580977117.8549250434395392"),
    ("63499233282.0282019728", "1312.31233"),
    ("-123123123123", "-1"),
];

const ROOT_OPERANDS: [(&str, &str); 4] = [
    ("57896044618658097711785492504343953926634992332820282019728.792003956564819967","17"),
    ("12379879872423987.123123123", "13"),
    ("12379879872423987.123123123", "5"),
    ("9", "2"),
];

const POW_OPERANDS: [(&str, &str); 4] = [
    ("12.123123123", "13"),
    ("1.123123123", "5"),
    ("4", "5"),
    ("9", "2"),
];

const TO_STRING_OPERANDS: [&str; 4] = [
    "57896044618658097711785492504343953926634992332820282019728.792003956564819967",
    "-11237987890123090890328.1928379813",
    "12379879872423987.123123123",
    "9",
];

const FROM_STRING_OPERANDS: [&str; 4] = [
    "57896044618658097711785492504343953926634992332820282019728.792003956564819967",
    "-11237987890123090890328.1928379813",
    "12379879872423987.123123123",
    "9",
];

bench_ops!(Decimal, "add");
bench_ops!(Decimal, "sub");
bench_ops!(Decimal, "mul");
bench_ops!(Decimal, "div");
bench_ops!(Decimal, "root", u32);
bench_ops!(Decimal, "pow", i64);
bench_ops!(Decimal, "to_string");
bench_ops!(Decimal, "from_string");
/*
criterion_group!(
    bench_decimal,
    bench_decimal_add,
    bench_decimal_sub,
    bench_decimal_mul,
    bench_decimal_div,
    bench_decimal_root,
    bench_decimal_pow,
    bench_decimal_to_string,
    bench_decimal_from_string
);
*/
//criterion_main!(bench_decimal);
