
use std::{fs::File, io::{self, BufRead, Write}, env};

use alea_core::prelude::{Instruction, InstVariant, CmpFlags};


fn main() {
    let args: Vec<String> = env::args().collect();

    let infilename = &args[1];
    let outfilename = &args[2];
    let mut outfile = File::create(outfilename).unwrap();

    let mut index = 0;
    for line in io::BufReader::new(File::open(infilename).unwrap()).lines() {
        let l = line.unwrap();
        if l.trim().starts_with("#") { continue; }

        let mut split = l.split('\t');
        let offset = split.next();
        match offset {
            Some(str) if (!str.is_empty()) => {
                println!(">{}<",str);
                let number : u32 = u32::from_str_radix(str.trim().trim_end_matches(':'), 16).unwrap();
                if index > number {
                    panic!("index > number");
                }
                while index < number {
                    outfile.write_all(&[0]).unwrap();
                    index += 1;
                }
            }
            _ => {}
        }

        let cmpstr_ = split.next();
        let signstr_ = split.next();
        let varstr_ = split.next();
        match (cmpstr_, signstr_, varstr_) {
            (Some(cmpstr), Some(signstr), Some(varstr)) if !cmpstr.is_empty() && !signstr.is_empty() && !varstr.is_empty() => {
                let instvariant : InstVariant = ron::from_str(&varstr).unwrap_or_else(|err| {
                    panic!("failed to parse line: {}\n Error: {}", l, err);
                });
                let cmp : CmpFlags = cmpstr.try_into().unwrap();
                let sign = match signstr {
                    "+" => false,
                    "-" => true,
                    _ => { panic!("Invalid sign"); }
                };
                let instr = Instruction { cmp: cmp, sign: sign, var: instvariant };
                let opcode : u32 = instr.into();
                println!("{:08x}  {}  {:x}", index, instr, opcode);
                outfile.write_all(&opcode.to_le_bytes()).unwrap();
                index += 4;
            }
            (Some(cmpstr), Some(signstr), Some(varstr)) if cmpstr.is_empty() && signstr.is_empty() && varstr.is_empty() => {}
            (None, None, None) => {}
            _ => { panic!("Invalid line"); }
        }

        let immediatestr = split.next();
        match immediatestr {
            Some(str) if (!str.is_empty()) => {
                let number : u32 = u32::from_str_radix(str.trim_start_matches("0x"), 16).unwrap();
                outfile.write_all(&number.to_le_bytes()).unwrap();
                index += 4;
            }
            _ => {}
        };
    }

}
