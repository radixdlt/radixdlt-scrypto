
#[macro_export]
macro_rules! process_op {
    ($t:ty, $op:ident, $bid:ident, "to_string") => {
        let $bid = format!("{}_to_string", $op);
        let $op = (<$t>::from(*$op), "_");
    };
    ($t:ty, $op:ident, $bid:ident, "from_string") => {
        let $bid = format!("{}_from_string", $op);
        let $op = ($op, "_");
    };
    ($t:ty, $op:ident, $bid:ident, "root", $prim_t:ty) => {
        let first = <$t>::from($op.0);
        let second = $op.1.to_string().parse::<$prim_t>().unwrap();
        let $bid = format!("{}_and_{}", $op.0, $op.1);
        let $op = (first, second);
    };
    ($t:ty, $op:ident, $bid:ident, "pow", $prim_t:ty) => {
        let first = <$t>::from($op.0);
        let second = $op.1.to_string().parse::<$prim_t>().unwrap();
        let $bid = format!("{}_and_{}", $op.0, $op.1);
        let $op = (first, second);
    };
    ($t:ty, $op:ident, $bid:ident, $ops:literal) => {
        let first = <$t>::from($op.0);
        let second = <$t>::from($op.1);
        let $bid = format!("{}_and_{}", $op.0, $op.1);
        let $op = (first, second);
    };
}

#[macro_export]
macro_rules! bench_ops {
    ($t:ty, $ops:literal) => {
        paste::item! {
            pub fn [< bench_ $t:lower _ $ops >] (c: &mut Criterion) {
                let test_descr = concat!(stringify!($t), "_", $ops);
                let mut group = c.benchmark_group(test_descr);
                for op in [< $ops:upper _OPERANDS >].iter() {
                    process_op!($t, op, bid, $ops);
                    group.bench_with_input(BenchmarkId::from_parameter(bid), &op, | b, (o_first, o_second) | {
                        b.iter(|| {
                            [< $t:lower _ $ops>](*o_first, *o_second)
                        })
                    });
                }
            }
        }
    };
    ($t:ty, $ops:literal, $prim_t:ty) => {
        paste::item! {
            pub fn [< bench_ $t:lower _ $ops >] (c: &mut Criterion) {
                let test_descr = concat!(stringify!($t), "_", $ops);
                let mut group = c.benchmark_group(test_descr);
                for op in [< $ops:upper _OPERANDS >].iter() {
                    process_op!($t, op, bid, $ops, $prim_t);
                    group.bench_with_input(BenchmarkId::from_parameter(bid), &op, | b, (o_first, o_second) | {
                        b.iter(|| {
                            [< $t:lower _ $ops>](*o_first, *o_second)
                        })
                    });
                }
            }
        }
    };
}
