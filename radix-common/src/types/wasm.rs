pub type BufferId = u32;

#[repr(C)]
pub struct Buffer(pub u64);

impl Buffer {
    pub fn new(id: u32, len: u32) -> Self {
        Self((id as u64) << 32 | (len as u64))
    }

    pub fn id(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn len(&self) -> u32 {
        (self.0 & 0xffffffff) as u32
    }

    pub fn transmute_i64(n: i64) -> Self {
        Self(n as u64)
    }

    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
}

#[repr(C)]
pub struct Slice(pub u64);

impl Slice {
    pub fn new(ptr: u32, len: u32) -> Self {
        Self((ptr as u64) << 32 | (len as u64))
    }

    pub fn ptr(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn len(&self) -> u32 {
        (self.0 & 0xffffffff) as u32
    }

    pub fn transmute_i64(n: i64) -> Self {
        Self(n as u64)
    }

    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
}

#[cfg(feature = "scrypto")]
mod scrypto_buffer_functions {
    use super::*;

    #[macro_export]
    macro_rules! wasm_extern_c {
        (
            $(
                $(#[$meta:meta])*
                pub fn $fn_ident: ident ( $($arg_name: ident: $arg_type: ty),* $(,)? ) $(-> $rtn_type: ty)?;
            )*
        ) => {
            #[cfg(target_arch = "wasm32")]
            extern "C" {
                $(
                    $(#[$meta])*
                    pub fn $fn_ident ( $($arg_name: $arg_type),* ) $(-> $rtn_type)?;
                )*
            }

            $(
                #[cfg(not(target_arch = "wasm32"))]
                $(#[$meta])*
                pub unsafe fn $fn_ident ( $(_: $arg_type),* ) $(-> $rtn_type)? {
                    unimplemented!("Not implemented for non-wasm targets")
                }
            )*
        };
    }
    pub use wasm_extern_c;

    wasm_extern_c! {
        /// Consumes a buffer by copying the contents into the specified destination.
        pub fn buffer_consume(buffer_id: BufferId, destination_ptr: *mut u8);
    }

    pub fn copy_buffer(buffer: Buffer) -> Vec<u8> {
        let len = buffer.len() as usize;
        let mut vec = Vec::<u8>::with_capacity(len);
        unsafe {
            buffer_consume(buffer.id(), vec.as_mut_ptr());
            vec.set_len(len);
        };
        vec
    }

    pub fn forget_vec(vec: Vec<u8>) -> Slice {
        let ptr = vec.as_ptr() as usize;
        let len = vec.len();
        assert!(ptr <= 0xffffffff && len <= 0xffffffff);

        // Note that the memory used by the Vec is forever leaked.
        // However, it's not an issue since the wasm instance will be destroyed after engine
        // consuming the data.
        sbor::rust::mem::forget(vec);

        Slice::new(ptr as u32, len as u32)
    }
}
#[cfg(feature = "scrypto")]
pub use scrypto_buffer_functions::*;
