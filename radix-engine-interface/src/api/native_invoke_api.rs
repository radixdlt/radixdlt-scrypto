use crate::api::types::*;

pub trait ClientNativeInvokeApi<E> {
    fn call_native<N: SerializableInvocation>(&mut self, invocation: N) -> Result<N::Output, E>;
}
