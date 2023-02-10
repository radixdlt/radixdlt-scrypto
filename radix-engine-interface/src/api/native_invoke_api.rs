use crate::api::types::*;

pub trait ClientNativeInvokeApi<E> {
    fn call_native_raw(
        &mut self,
        fn_identifier: NativeFn,
        invocation: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn call_native<N: SerializableInvocation>(&mut self, invocation: N) -> Result<N::Output, E>;
}
