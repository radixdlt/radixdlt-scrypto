#![cfg_attr(not(feature = "std"), no_std)]

use sbor::*;

eager_replace! {
    [!EAGER:set:ident! #boo = Hello World 2]
    struct HelloWorld2 {}
    type [!EAGER:ident! X "Boo" [!EAGER:concat! Hello 1] #boo] = HelloWorld2;
}

#[test]
fn can_create() {
    let _x: XBooHello1HelloWorld2 = HelloWorld2 {};
}
