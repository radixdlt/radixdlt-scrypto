use scrypto::prelude::*;

blueprint! {
    struct Hello {
    }

    impl Hello {
        pub fn a(b: String) {
            info!("b: {}", b);
            let c = b.len();
            info!("c: {}", c);
            let d: U32 = b.parse::<U32>().unwrap();
            info!("d: {}", d);
            let e = U64::from(100u8);
            let f = e + d;
            info!("f: {}", f);
            let g = e - d;
            info!("g: {}", g);
            let h = e * d;
            info!("h: {}", h);
            let i = e / d;
            info!("i: {}", i);
            let j = e % d;
            info!("j: {}", j);
            let k = d.pow(2);
            info!("k: {}", k);
        }
    }
}

