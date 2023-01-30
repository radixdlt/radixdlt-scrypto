use crate::{system::kernel_modules::execution_trace::SysCallTrace, types::*};

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TrackedEvent {
    SysCallTrace(SysCallTrace),
}
