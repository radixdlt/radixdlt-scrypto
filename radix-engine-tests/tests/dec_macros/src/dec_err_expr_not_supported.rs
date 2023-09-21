use radix_engine_interface::prelude::*;

fn main() {
    // Invalid expression
    const X: Decimal = dec!(a);
    let _ = format!("{:?}", X);
}
