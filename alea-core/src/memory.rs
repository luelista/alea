use std::fmt::{Debug, Display};

use bytes::BytesMut;
use tracing::{trace, warn};

pub trait Memory {
    fn size(&self) -> u32;
    fn read(&self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, value: u8);

    fn read_u16(&self, addr: u32) -> u16 {
        let b1 = self.read(addr);
        let b2 = self.read(addr + 1);

        u16::from_le_bytes([b1, b2])
    }

    fn read_u32(&self, addr: u32) -> u32 {
        let b1 = self.read(addr);
        let b2 = self.read(addr + 1);
        let b3 = self.read(addr + 2);
        let b4 = self.read(addr + 3);

        u32::from_le_bytes([b1, b2, b3, b4])
    }

    fn write_u16(&mut self, addr: u32, value: u16) {
        let [b1, b2] = value.to_le_bytes();
        self.write(addr, b1);
        self.write(addr + 1, b2);
    }

    fn write_u32(&mut self, addr: u32, value: u32) {
        let [b1, b2, b3, b4] = value.to_le_bytes();
        self.write(addr, b1);
        self.write(addr + 1, b2);
        self.write(addr + 2, b3);
        self.write(addr + 3, b4);
    }

    fn pre_step(&mut self) {}
    fn post_step(&mut self) {
        warn!("mem-trait default: post-step");
    }
}

type BoxedMem = Box<dyn Memory>;

impl Memory for BytesMut {
    fn size(&self) -> u32 {
        self.len() as u32
    }

    fn read(&self, addr: u32) -> u8 {
        self[addr as usize]
    }

    fn write(&mut self, addr: u32, value: u8) {
        self.get_mut(addr as usize).map(|v| *v = value);
    }
}

#[derive(Default)]
pub struct MemoryMap {
    pub mems: Vec<MappedMemory>,
}

impl MemoryMap {
    fn pos_resp(&self, addr: u32) -> Option<usize> {
        self.mems
            .iter()
            .position(|mem| mem.start <= addr && mem.end >= addr)
    }

    fn get_responsible(&self, addr: u32) -> Option<&MappedMemory> {
        self.pos_resp(addr).map(|idx| &self.mems[idx])
    }

    fn get_responsible_mut(&mut self, addr: u32) -> Option<&mut MappedMemory> {
        self.pos_resp(addr).map(|idx| &mut self.mems[idx])
    }

    fn collides(&self, b: &MappedMemory) -> bool {
        self.mems.iter().any(|a| {
            if a.start == b.start || a.end == b.end {
                false
            } else if a.start < b.start {
                a.end > b.start
            } else {
                a.start < b.end
            }
        })
    }

    pub fn add(&mut self, mem: impl Memory + 'static, at: u32) -> Result<(), ()> {
        let mapped = MappedMemory::new(Box::new(mem), at);
        if self.collides(&mapped) {
            return Err(());
        }
        self.mems.push(mapped);

        Ok(())
    }
}

impl Display for MemoryMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemoryMap({:?})", self.mems)
    }
}

impl Memory for MemoryMap {
    fn size(&self) -> u32 {
        u32::MAX
    }

    fn read(&self, addr: u32) -> u8 {
        match self.get_responsible(addr) {
            None => {
                warn!("out of bounds read at {:#010x}", addr);
                0
            }
            Some(mem) => {
                trace!("mmap: read of {:#x} delegated to {:?}", addr, mem);
                mem.read(addr)
            }
        }
    }

    fn write(&mut self, addr: u32, value: u8) {
        self.get_responsible_mut(addr)
            .map(|mem| mem.write(addr, value));
    }
}

pub struct MappedMemory {
    pub start: u32,
    pub end: u32,
    pub inner: BoxedMem,
}

impl MappedMemory {
    pub fn new(mem: BoxedMem, start: u32) -> MappedMemory {
        let size = mem.size();
        let end = start + size - 1;

        Self {
            start,
            end,
            inner: mem,
        }
    }
}

impl Debug for MappedMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MappedMem({:#x} .. {:#x})", self.start, self.end)
    }
}

impl Memory for MappedMemory {
    fn size(&self) -> u32 {
        self.inner.size()
    }

    fn read(&self, addr: u32) -> u8 {
        let internal = addr - self.start;
        assert!(internal < self.size());
        self.inner.read(internal)
    }

    fn write(&mut self, addr: u32, value: u8) {
        let internal = addr - self.start;
        assert!(internal < self.size());
        self.inner.write(internal, value);
    }
}
