use radix_engine::engine::*;
use scrypto::prelude::*;

#[test]
fn test_liquidity_pool() {
    // Create an in-memory Radix Engine.
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();
    let mut proc = runtime.start_process(true);

    // Publish this package.
    let package = proc.publish(package_code!()).unwrap();

    // Creating TokenA
    /*let token_a: Bucket = ResourceBuilder::new()
    .metadata("name", "Token A")
    .metadata("symbol", "tokenA")
    .create_fixed(1000);
    */
    let token_a: Address = proc
        .call_function(package, "Token", "new", args!("Token A", "tokenA"))
        .and_then(decode_return)
        .unwrap();

    // Invoke the `get_vault` function.
    let vault_amount_a: Amount = proc
        .call_method(token_a, "get_vault_amount", args!())
        .and_then(decode_return)
        .unwrap();  

    assert_eq!(1000, vault_amount_a.as_u32());

    // Creating TokenB
    let token_b: Address = proc
        .call_function(package, "Token", "new", args!("Token B", "tokenB"))
        .and_then(decode_return)
        .unwrap();

    // Invoke the `get_vault_amount` function.
    let vault_amount_b: Amount = proc
        .call_method(token_b, "get_vault_amount", args!())
        .and_then(decode_return)
        .unwrap();  

    assert_eq!(1000, vault_amount_b.as_u32());

    // Creating Liquidity Pool
    let liquidity_pool: Address = proc
        .call_function(package, "LiquidityPool", "new", args!(token_a, token_b))
        .and_then(decode_return)
        .unwrap();

    let bid: BID = proc.create_bucket(3.into(), token_b);

    // Swapping
   let vaults: Result<(Address, Address), radix_engine::execution::RuntimeError> = proc
        .call_method(liquidity_pool, "swap", args!(Amount::from(1), Bucket::from(bid)))
        .and_then(decode_return);

    match vaults {
        Ok(a) => print! ("It worked!"),
        Err(err) => print! ("There was an error {}", err)
    }
        /*
    // Creating TokenA
    let token_a: Address = proc
        .call_function(package, "TokenA", "new", args!())
        .and_then(decode_return)
        .unwrap();    

    // Invoke the `say_hello` function.
    let rtn: u32 = proc
        .call_method(component, "say_hello", args!())
        .and_then(decode_return)
        .unwrap();
    assert_eq!(1, rtn);
    */
}
