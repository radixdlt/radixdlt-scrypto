#![cfg_attr(not(feature = "std"), no_std)]

use sbor::*;

eager_replace! {
    [!SET! #bytes = 32]
    [!SET! #postfix = Hello World #bytes]
    [!SET:raw! #MyRawVar = Test no #str [!ident! replacement]]
    struct MyStruct;
    type [!ident! X "Boo" [!concat! Hello 1] #postfix] = MyStruct;
    const MY_NUM: u32 = [!literal! 1337u #bytes];
    const MY_STR: &'static str = [!stringify! #MyRawVar];
}

#[test]
fn complex_example_evaluates_correctly() {
    let _x: XBooHello1HelloWorld32 = MyStruct;
    assert_eq!(MY_NUM, 1337u32);
    // Note: TokenStream loses information about whether idents are attached to the proceeding punctuation.
    // So this is an equivalent raw stream to #MyRawVar.
    assert_eq!(MY_STR, "Test no # str [! ident! replacement]");
}
