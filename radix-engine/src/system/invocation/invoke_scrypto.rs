use crate::types::*;
use radix_engine_interface::api::types::ScryptoInvocation;
use radix_engine_interface::api::ClientStaticInvokeApi;

pub fn invoke_scrypto_fn<Y, E>(
    invocation: ScryptoInvocation,
    api: &mut Y,
) -> Result<IndexedScryptoValue, E>
where
    Y: ClientStaticInvokeApi<E>,
{
    let rtn = api.invoke(invocation)?;
    Ok(IndexedScryptoValue::from_value(rtn))
}
