use crate::{system::kernel_modules::execution_trace::KernelCallTrace, types::*};

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TrackedEvent {
    KernelCallTrace(KernelCallTrace),
}
