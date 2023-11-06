
use std::{fs::File, io::{self, BufRead, Write}, env};

use alea_core::prelude::Instruction;


fn main() {
    let args: Vec<String> = env::args().collect();

    let infilename = &args[1];
    let outfilename = &args[2];
    let mut outfile = File::create(outfilename).unwrap();

    let mut index = 0;
    for line in io::BufReader::new(File::open(infilename).unwrap()).lines() {
        let l = line.unwrap();
        if l.starts_with("0x") {
            let number : u32 = u32::from_str_radix(l.trim_start_matches("0x"), 16).unwrap();
            outfile.write_all(&number.to_le_bytes()).unwrap();
            index += 4;
        } else if l.starts_with(".org ") {
            let number : u32 = u32::from_str_radix(l.trim_start_matches(".org "), 16).unwrap();
            if index > number {
                panic!("index > number");
            }
            while index < number {
                outfile.write_all(&[0]).unwrap();
                index += 1;
            }
        } else if l.starts_with("{") {
            let instr : Instruction = serde_json::from_str(&l).unwrap_or_else(|err| {
                panic!("failed to parse line: {}\n Error: {}", l, err);
            });
            let opcode : u32 = instr.into();
            println!("{:08x}  {}  {:x}", index, instr, opcode);
            outfile.write_all(&opcode.to_le_bytes()).unwrap();
            index += 4;
        }
    }

}
