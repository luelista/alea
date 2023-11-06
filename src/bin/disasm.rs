
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
        if instr_word == 0 {
            let mut zeroidx = index;
            while zeroidx < program.size() && program.read_u32(zeroidx + 4) == 0 {
                zeroidx += 4;
            }
            if zeroidx - index > 16 {
                index = zeroidx + 4;
                print!("\n\n{:04x}: ", index);
                continue;
            }
        }
        
        match <u32 as TryInto<Instruction>>::try_into(instr_word) {
            Ok(instr) => {
                match instr.var {
                    InstVariant::Nop | InstVariant::Halt => {
                        if instr_word != 0 && (instr_word & 0xf8ffffff) != 1 {
                            println!("\t\t\t\t0x{:08x}", instr_word);
                            index += 4;
                            continue;
                        }
                    }
                    _ => {}
                }
                let json = ron::to_string(&instr.var).unwrap();
                println!("\t{}\t{}\t{}\t{}", Into::<&str>::into(instr.cmp), if instr.sign { "-" } else {"+"}, json,
                if instr.has_upcoming_value() {
                    index += 4;
                    let data_word = program.read_u32(index);
                    format!("0x{:08x}", data_word)
                } else { String::default() });
                
            }
            Err(_err) => {
                println!("\t\t\t\t0x{:08x}", instr_word);
            }
        }

        index += 4;
    }

}
