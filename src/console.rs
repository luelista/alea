use alea_core::prelude::*;
use bytes::BytesMut;

pub struct Console(pub BytesMut);

impl Memory for Console {
    fn size(&self) -> u32 {
        self.0.size()
    }

    fn read(&self, addr: u32) -> u8 {
        self.0.read(addr)
    }

    fn write(&mut self, addr: u32, value: u8) {
        self.0.write(addr, value)
    }

    fn post_step(&mut self) {
        if self.ready_to_flush() {
            self.flush();
        }
    }
}

impl Console {
    fn ready_to_flush(&self) -> bool {
        self.0.iter().any(|thing| *thing == 0x17)
    }

    fn flush(&mut self) {
        let mut bytes = Vec::new();
        for byte in self.0.iter_mut() {
            let copy = *byte;
            *byte = 0;
            if copy == 0x17 {
                break;
            }
            bytes.push(copy);
        }
        let s = String::from_utf8_lossy(bytes.as_slice());
        print!("{}", s);
    }
}
