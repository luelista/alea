
use std::{fs::File, io::{self, BufRead, Write}, env};

use alea_core::prelude::Instruction;


fn main() {
    let args: Vec<String> = env::args().collect();

    let infilename = &args[1];

    for line in io::BufReader::new(File::open(infilename).unwrap()).lines() {
        let l = line.unwrap();
        if l.starts_with("0x") {
            println!("\t\t\t\t{}", l);
        } else if l.starts_with(".org ") {
            println!("{}:", l.trim_start_matches(".org "));
        } else if l.starts_with("{") {
            let instr : Instruction = serde_json::from_str(&l).unwrap_or_else(|err| {
                panic!("failed to parse line: {}\n Error: {}", l, err);
            });
            let json = ron::to_string(&instr.var).unwrap();
            println!("\t{}\t{}\t{}\t", Into::<&str>::into(instr.cmp), if instr.sign { "-" } else {"+"}, json);
            
        } else {
            println!("{}", l);
        }
    }

}
