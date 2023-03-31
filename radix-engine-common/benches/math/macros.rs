// set it to false if you want full benchmarks.
pub const QUICK: bool = true;

#[macro_export]
macro_rules! ops_fn {
    ($t:ty, $pow_fn:ident, $exp_t:ty) => {
        paste::item! {
            fn [< $t:lower _add >](a: &$t, b: &$t) -> $t {
                a + b
            }

            fn [< $t:lower _sub >](a: &$t, b: &$t) -> $t {
                a - b
            }

            fn [< $t:lower _mul >](a: &$t, b: &$t) -> $t {
                a * b
            }

            fn [< $t:lower _div >](a: &$t, b: &$t) -> $t {
                a / b
            }

            fn [< $t:lower _pow >](a: &$t, exp: &$exp_t) -> $t {
                a.$pow_fn(*exp)
            }

            fn [< $t:lower _to_string >](a: &$t, _: &str) -> String {
                a.to_string()
            }

            fn [< $t:lower _from_string >](s: &str, _: &str) -> $t {
                <$t>::from_str(s).unwrap()
            }
        }
    };
    ($t:ty, $pow_fn:ident, $exp_t:ty, "clone") => {
        paste::item! {
            fn [< $t:lower _add >](a: &$t, b: &$t) -> $t {
                a.clone() + b.clone()
            }

            fn [< $t:lower _sub >](a: &$t, b: &$t) -> $t {
                a.clone() - b.clone()
            }

            fn [< $t:lower _mul >](a: &$t, b: &$t) -> $t {
                a.clone() * b.clone()
            }

            fn [< $t:lower _div >](a: &$t, b: &$t) -> $t {
                a.clone() / b.clone()
            }

            fn [< $t:lower _pow >](a: &$t, exp: &$exp_t) -> $t {
                a.clone().$pow_fn(*exp)
            }

            fn [< $t:lower _to_string >](a: &$t, _: &str) -> String {
                a.to_string()
            }

            fn [< $t:lower _from_string >](s: &str, _: &str) -> $t {
                <$t>::from_str(s).unwrap()
            }
        }
    };
}

#[macro_export]
macro_rules! ops_root_fn {
    ($t:ty, $root_fn:ident) => {
        paste::item! {
            fn [< $t:lower _root >](a: &$t, n: &u32)  {
                let _ = a.$root_fn(*n);
            }
        }
    };
    ($t:ty, $root_fn:ident, "clone") => {
        paste::item! {
            fn [< $t:lower _root >](a: &$t, n: &u32)  {
                let _ = a.clone().$root_fn(*n);
            }
        }
    };
}

#[macro_export]
macro_rules! process_op {
    ($t:ty, $i:ident, $op:ident, $bid:ident, "to_string") => {
        let $bid = format!("{}", $i);
        let $op = (<$t>::from_str(*$op).unwrap(), "_");
    };
    ($t:ty, $i:ident, $op:ident, $bid:ident, "from_string") => {
        let $bid = format!("{}", $i);
        let $op = ($op, "_");
    };
    ($t:ty, $i:ident, $op:ident, $bid:ident, "root", $prim_t:ty) => {
        let first = <$t>::from_str($op.0).unwrap();
        let second = $op.1.to_string().parse::<$prim_t>().unwrap();
        let $bid = format!("{}", $i);
        let $op = (first, second);
    };
    ($t:ty, $i:ident, $op:ident, $bid:ident, "pow", $prim_t:ty) => {
        let first = <$t>::from_str($op.0).unwrap();
        let second = $op.1.to_string().parse::<$prim_t>().unwrap();
        let $bid = format!("{}", $i);
        let $op = (first, second);
    };
    ($t:ty, $i:ident, $op:ident, $bid:ident, $ops:literal) => {
        let first = <$t>::from_str($op.0).unwrap();
        let second = <$t>::from_str($op.1).unwrap();
        let $bid = format!("{}", $i);
        let $op = (first, second);
    };
}

#[macro_export]
macro_rules! bench_ops {
    ($t:ty, $ops:literal, "no_ref") => {
        paste::item! {
            pub fn [< bench_ $t:lower _ $ops >] (c: &mut Criterion) {
                let group_name = concat!(stringify!($t), "::", $ops);
                let mut group = c.benchmark_group(group_name);
                for (i, op) in [< $ops:upper _OPERANDS >].iter().enumerate() {
                    process_op!($t, i, op, bid, $ops);
                    group.bench_with_input(BenchmarkId::from_parameter(bid), &op, | b, (o_first, o_second) | {
                        b.iter(|| {
                            [< $t:lower _ $ops>](*o_first, *o_second)
                        })
                    });
                    if QUICK {
                        break
                    }
                }
            }
        }
    };
    ($t:ty, $ops:literal) => {
        paste::item! {
            pub fn [< bench_ $t:lower _ $ops >] (c: &mut Criterion) {
                let group_name = concat!(stringify!($t), "::", $ops);
                let mut group = c.benchmark_group(group_name);
                for (i, op) in [< $ops:upper _OPERANDS >].iter().enumerate() {
                    process_op!($t, i, op, bid, $ops);
                    group.bench_with_input(BenchmarkId::from_parameter(bid), &op, | b, (o_first, o_second) | {
                        b.iter(|| {
                            [< $t:lower _ $ops>](&*o_first, &*o_second)
                        })
                    });
                    if QUICK {
                        break
                    }
                }
            }
        }
    };
    ($t:ty, $ops:literal, $prim_t:ty) => {
        paste::item! {
            pub fn [< bench_ $t:lower _ $ops >] (c: &mut Criterion) {
                let group_name = concat!(stringify!($t), "::", $ops);
                let mut group = c.benchmark_group(group_name);
                for (i, op) in [< $ops:upper _OPERANDS >].iter().enumerate() {
                    process_op!($t, i, op, bid, $ops, $prim_t);
                    group.bench_with_input(BenchmarkId::from_parameter(bid), &op, | b, (o_first, o_second) | {
                        b.iter(|| {
                            [< $t:lower _ $ops>](&*o_first, &*o_second)
                        })
                    });
                    if QUICK {
                        break
                    }
                }
            }
        }
    };
}
