#![cfg_attr(not(feature = "std"), no_std)]

use sbor::*;

eager_replace! {
    [!SET! #bytes = 32]
    [!SET! #postfix = Hello World #bytes]
    [!SET:raw! #MyRawVar = Test no #str [!ident! replacement]]
    struct MyStruct;
    type [!ident! X "Boo" [!concat! Hello 1] #postfix] = MyStruct;
    const MY_NUM: u32 = [!literal! 1337u #bytes];
    const MY_CONCAT: &'static str = [!concat! #MyRawVar];
    const MY_STRINGIFY: &'static str = [!stringify! #MyRawVar];
}

#[test]
fn complex_example_evaluates_correctly() {
    let _x: XBooHello1HelloWorld32 = MyStruct;
    assert_eq!(MY_NUM, 1337u32);
    assert_eq!(MY_CONCAT, "Testno#str[!ident!replacement]");
    // Note: TokenStream loses information about whether idents are attached to the proceeding punctuation...
    // And actually the exact string is compiler-version dependent!
    //
    // Both of the following have been seen on different test runs on develop/CI:
    // * `"Test no # str [! ident! replacement]"`
    // * `"Test no # str [!ident! replacement]"`
    //
    // So instead, let's remove spaces and check it's equivalent to flatten_concat:
    assert_eq!(MY_STRINGIFY.replace(" ", ""), MY_CONCAT);
}
