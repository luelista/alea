mod console;

use alea_core::prelude::*;

use bytes::BytesMut;
use console::Console;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut cpu = CPU::default();

    let hello_world_data = BytesMut::from(b"hello, world!\n\x17".as_slice());
    let program = BytesMut::from(include_bytes!("hello.alea").as_slice());
    let mem = BytesMut::zeroed(0x4000);
    let console = Console(BytesMut::zeroed(0x20));

    cpu.mmap.add(program, 0x0).unwrap();
    cpu.mmap.add(hello_world_data, 0x2000).unwrap();
    cpu.mmap.add(mem, 0x4000).unwrap();
    cpu.mmap.add(console, 0x8000).unwrap();

    cpu.step_wait = false;

    info!("running with {}", cpu.mmap);

    cpu.run_until_halt().unwrap();
}
