use scrypto::prelude::*;

#[blueprint]
mod hello {
    struct Hello {}

    impl Hello {
        pub fn native_and_safe_integer_interop(a: U32) -> U32 {
            a
        }

        pub fn integer_basic_ops(b: String) {
            info!("b: {}", b);
            let c = b.len();
            info!("c: {}", c);
            let d: U32 = b.parse::<U32>().unwrap();
            info!("d: {}", d);
            let e = U64::from(100u8);
            let f = e + U64::from(d);
            info!("f: {}", f);
            let g = e - U64::from(d);
            info!("g: {}", g);
            let h = e * U64::from(d);
            info!("h: {}", h);
            let i = e / U64::from(d);
            info!("i: {}", i);
            let j = e % U64::from(d);
            info!("j: {}", j);
            let k = d.pow(2);
            info!("k: {}", k);
        }
    }
}
