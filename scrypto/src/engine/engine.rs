use crate::engine::types

pub struct RadixEngine {
    packages: Vec<String>,
    components: Vec<String>,
    resource_defs: Vec<String>,
}

impl RadixEngine { 
    pub fn get_resource_def(&mut self, id: usize) -> &mut String {
        &mut self.a[id]
    }
}

static mut SYSTEM: RadixEngine = RadixEngine { a: Vec::new() };

pub fn engine() -> &'static mut RadixEngine {
    unsafe { &mut SYSTEM }
}

fn main() {
    engine().create_resource_def();
    let resource_def = engine().get_resource_def(self.resource_def_id);
}
