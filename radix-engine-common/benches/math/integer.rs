use std::{concat, str::FromStr};

use bnum::BInt;
use criterion::{BenchmarkId, Criterion};
use ethnum::I256 as EthnumI256;
use num_bigint::BigInt;
use num_integer::Roots;
use rug::{ops::Pow as RugPow, Integer};

use crate::macros::QUICK;
use crate::{bench_ops, ops_fn, ops_root_fn, process_op};

type BnumBint256 = BInt<4>;

const ADD_OPERANDS: [(&str, &str); 4] = [
    (
        "278960446186580977117854925043439539266349000000000000000000000000000000000",
        "278960446186580977117854925043439539266349000000000000000000000000000000000",
    ),
    (
        "-278960446186580977117854925043439539266349000000000000000000000000000000000",
        "278960446186580977117854925043439539266349000000000000000000000000000000000",
    ),
    ("1", "-1"),
    (
        "-278960446186580977117854925043439539266349000000000000000000000000000000000",
        "-278960446186580977117854925043439539266349000000000000000000000000000000000",
    ),
];

const SUB_OPERANDS: [(&str, &str); 4] = [
    (
        "278960446186580977117854925043439539266349000000000000000000000000000000000",
        "278960446186580977117854925043439539266349000000000000000000000000000000000",
    ),
    (
        "-278960446186580977117854925043439539266349000000000000000000000000000000000",
        "278960446186580977117854925043439539266349000000000000000000000000000000000",
    ),
    ("1", "-1"),
    (
        "-278960446186580977117854925043439539266349000000000000000000000000000000000",
        "-278960446186580977117854925043439539266349000000000000000000000000000000000",
    ),
];

const MUL_OPERANDS: [(&str, &str); 4] = [
    (
        "278960446186580977117854925043439539",
        "2789604461865809771178549250434395392",
    ),
    (
        "-278960446186580977117854925043439539",
        "2789604461865809771178549250434395392",
    ),
    ("634992332820282019728", "131231233"),
    ("-123123123123", "-1"),
];

const DIV_OPERANDS: [(&str, &str); 4] = [
    (
        "278960446186580977117854925043439539",
        "2789604461865809771178549250434395392",
    ),
    (
        "-278960446186580977117854925043439539",
        "2789604461865809771178549250434395392",
    ),
    ("634992332820282019728", "131231233"),
    ("-123123123123", "-1"),
];

const ROOT_OPERANDS: [(&str, &str); 4] = [
    (
        "57896044618658097711785492504343953926634992332820282019728",
        "17",
    ),
    ("12379879872423987", "13"),
    ("12379879872423987", "5"),
    ("9", "2"),
];

const POW_OPERANDS: [(&str, &str); 4] = [("12", "13"), ("1123123123", "5"), ("4", "5"), ("9", "2")];

const TO_STRING_OPERANDS: [&str; 4] = [
    "578960446186580977117854925043439539266349923328202820197792003956564819967",
    "-112379878901230908903281928379813",
    "12379879872423987123123123",
    "9",
];

const FROM_STRING_OPERANDS: [&str; 4] = [
    "578960446186580977117854925043439539266349923328202820197792003956564819967",
    "-112379878901230908903281928379813",
    "12379879872423987123123123",
    "9",
];

ops_fn!(BigInt, pow, u32);
ops_root_fn!(BigInt, nth_root);
bench_ops!(BigInt, "add");
bench_ops!(BigInt, "sub");
bench_ops!(BigInt, "mul");
bench_ops!(BigInt, "div");
bench_ops!(BigInt, "root", u32);
bench_ops!(BigInt, "pow", u32);
bench_ops!(BigInt, "to_string");
bench_ops!(BigInt, "from_string");

ops_fn!(Integer, pow, u32, "clone");
ops_root_fn!(Integer, root, "clone");
bench_ops!(Integer, "add");
bench_ops!(Integer, "sub");
bench_ops!(Integer, "mul");
bench_ops!(Integer, "div");
bench_ops!(Integer, "root", u32);
bench_ops!(Integer, "pow", u32);
bench_ops!(Integer, "to_string");
bench_ops!(Integer, "from_string");

ops_fn!(BnumBint256, pow, u32);
ops_root_fn!(BnumBint256, nth_root);
bench_ops!(BnumBint256, "add");
bench_ops!(BnumBint256, "sub");
bench_ops!(BnumBint256, "mul");
bench_ops!(BnumBint256, "div");
bench_ops!(BnumBint256, "root", u32);
bench_ops!(BnumBint256, "pow", u32);
bench_ops!(BnumBint256, "to_string");
bench_ops!(BnumBint256, "from_string");

ops_fn!(EthnumI256, pow, u32);
bench_ops!(EthnumI256, "add");
bench_ops!(EthnumI256, "sub");
bench_ops!(EthnumI256, "mul");
bench_ops!(EthnumI256, "div");
// Ethnum does not implement root function
bench_ops!(EthnumI256, "pow", u32);
bench_ops!(EthnumI256, "to_string");
bench_ops!(EthnumI256, "from_string");
