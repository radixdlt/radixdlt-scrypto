use crate::{system::kernel_modules::trace::SysCallTrace, types::*};

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TrackedEvent {
    SysCallTrace(SysCallTrace),
}
