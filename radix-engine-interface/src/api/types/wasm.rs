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
