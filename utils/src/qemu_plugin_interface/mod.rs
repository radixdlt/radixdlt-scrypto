use std::os::unix::net::UnixDatagram;

const SRV_SOCKET_FN: &str = "/tmp/scrypto-qemu-plugin-server.socket";
const CLI_SOCKET_FN: &str = "/tmp/scrypto-qemu-plugin-client.socket";


std::thread_local! {
    pub static QEMU_PLUGIN: std::cell::RefCell<QemuPluginInterface> = std::cell::RefCell::new(QemuPluginInterface::new(true));
}

pub struct QemuPluginInterface {
    enabled: bool,
    counters_stack: Vec<(String,u64)>,
    stack_top: usize,
    socket: UnixDatagram
}

impl QemuPluginInterface {
    pub fn new(enabled: bool) -> Self {

        std::fs::remove_file(CLI_SOCKET_FN).unwrap_or_default();

        let socket = UnixDatagram::bind(CLI_SOCKET_FN).unwrap();
        socket.set_read_timeout(None).unwrap();

        let mut ret = Self {
            enabled,
            counters_stack: Vec::with_capacity(100),
            stack_top: 0,
            socket
        };

        for _ in 0..ret.counters_stack.capacity() {
            ret.counters_stack.push((String::with_capacity(50),0));
        }

        ret
    }

    pub fn get_current_stack(&self) -> usize {
        self.stack_top
    }

    pub fn start_counting(&mut self, key: &str) {
        if !self.enabled {
            return;
        }

        if self.stack_top == self.counters_stack.len() {
            panic!("Stack too small");
        }

        self.counters_stack[self.stack_top].0.push_str(key);

        let n = self.connect(SRV_SOCKET_FN);

        self.counters_stack[self.stack_top].1 = n;

        self.stack_top += 1;
    }

    pub fn stop_counting(&mut self) -> (usize, u64) {
        if !self.enabled {
            return (0,0);
        }

        if self.stack_top == 0 {
            panic!("Not counting!");
        }
        self.stack_top -= 1;

        let n = self.connect(SRV_SOCKET_FN);

        self.counters_stack[self.stack_top].1 = n - self.counters_stack[self.stack_top].1;

        let ret = self.counters_stack[self.stack_top].1;
        (self.stack_top, ret)
    }

    fn connect(&mut self, addr: &str) -> u64 {

        self.socket.send_to(b"", addr).unwrap();
        //let mut buf = Vec::with_capacity(64);
        let mut buf = [0; 100];
        //self.socket.recv(&mut buf)
        let (_count, _address) = self.socket.recv_from(&mut buf).unwrap();

        let ret = u64::from_be_bytes(buf[..8].try_into().unwrap());

        //println!("socket {:?} sent {:?} -> {}", address, &buf[..count], ret);
        //let s = [0..ret].map(|_| " ").collect::<String>();
        //let s = String::from_utf8(vec![b' '; ret as usize]).unwrap();
        ret

    }
}
