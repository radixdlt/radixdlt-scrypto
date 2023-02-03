use crate::types::*;
use radix_engine_interface::api::ClientStaticInvokeApi;

use super::invoke_native::invoke_native_fn;
use super::invoke_scrypto::invoke_scrypto_fn;

pub fn invoke_call_table<Y, E>(
    invocation: CallTableInvocation,
    api: &mut Y,
) -> Result<IndexedScryptoValue, E>
where
    Y: ClientStaticInvokeApi<E>,
{
    let return_data = match invocation {
        CallTableInvocation::Native(native) => {
            IndexedScryptoValue::from_typed(invoke_native_fn(native, api)?.as_ref())
        }
        CallTableInvocation::Scrypto(scrypto) => invoke_scrypto_fn(scrypto, api)?,
    };

    Ok(return_data)
}
