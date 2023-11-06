
use std::{fs::File, io::Read, env};

use alea_core::prelude::*;

use bytes::BytesMut;

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];

	let mut buffer = Vec::new();
	let mut file = File::open(path).unwrap();

	file.read_to_end(&mut buffer).unwrap();

    let program = BytesMut::from(buffer.as_slice());

    let mut index: u32 = 0;

    while index < program.size() {
        let instr_word = program.read_u32(index);
        let instr : Instruction = <u32 as TryInto<Instruction>>::try_into(instr_word).unwrap();
        println!("# {:04x}: ", index);
        let json = serde_json::to_string(&instr).unwrap();
        println!("{}", json);
        if instr.has_upcoming_value() {
            index += 4;
            let data_word = program.read_u32(index);
            println!("# {:04x}: ", index);
            println!("0x{:08x}", data_word);
            
        }

        index += 4;
    }

}
