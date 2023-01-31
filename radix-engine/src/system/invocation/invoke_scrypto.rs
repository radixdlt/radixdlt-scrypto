use crate::types::*;
use radix_engine_interface::api::types::ScryptoInvocation;
use radix_engine_interface::api::{ClientComponentApi, ClientPackageApi};

pub fn invoke_scrypto_fn<Y, E>(invocation: ScryptoInvocation, api: &mut Y) -> Result<Vec<u8>, E>
where
    Y: ClientPackageApi<E> + ClientComponentApi<E>,
{
    if let Some(receiver) = invocation.receiver {
        api.call_method(receiver, &invocation.fn_name, invocation.args)
    } else {
        api.call_function(
            invocation.package_address,
            &invocation.blueprint_name,
            &invocation.fn_name,
            invocation.args,
        )
    }
}
