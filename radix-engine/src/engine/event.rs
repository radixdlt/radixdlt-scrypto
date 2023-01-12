use crate::model::SysCallTrace;
use crate::types::*;

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TrackedEvent {
    SysCallTrace(SysCallTrace),
}
