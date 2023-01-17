use crate::api::Invocation;
use crate::data::ScryptoDecode;
use crate::model::CallTableInvocation;

pub trait SerializableInvocation:
    Into<CallTableInvocation> + Invocation<Output = Self::ScryptoOutput>
{
    type ScryptoOutput: ScryptoDecode;
}

pub type BufferId = u32;
pub type Buffer = u64;

#[macro_export]
macro_rules! buffer_id {
    ($buf: expr) => {
        ($buf >> 32) as u32
    };
}

#[macro_export]
macro_rules! buffer_len {
    ($buf: expr) => {
        ($buf & 0xffffffff) as usize
    };
}

#[macro_export]
macro_rules! buffer {
    ($id: expr, $len: expr) => {
        (($id as u64) << 32) | ($len as u64)
    };
}

pub type Slice = u64;

#[macro_export]
macro_rules! return_data_ptr {
    ($buf: expr) => {
        ($buf >> 32) as usize
    };
}

#[macro_export]
macro_rules! return_data_len {
    ($buf: expr) => {
        ($buf & 0xffffffff) as usize
    };
}
