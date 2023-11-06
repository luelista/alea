
use std::{fs::File, io::{self, BufRead, Write}, env};

use alea_core::prelude::{Instruction, InstVariant, CmpFlags};


fn main() {
    let args: Vec<String> = env::args().collect();

    let infilename = &args[1];
    let outfilename = &args[2];
    let mut outfile = File::create(outfilename).unwrap();

    let mut index = 0;
    let mut linum = 0;
    for line in io::BufReader::new(File::open(infilename).unwrap()).lines() {
        linum += 1;
        let l = line.unwrap();
        match (||{
            if l.trim().starts_with("#") { return Ok(()); }

            let mut split = l.split('\t');
            let offset = split.next();
            match offset {
                Some(str) if (!str.is_empty()) => {
                    println!(">{}<",str);
                    let number : u32 = u32::from_str_radix(str.trim().trim_end_matches(':'), 16).map_err(|err| err.to_string())?;
                    if index > number {
                        return Err(format!("current write offset 0x{:x} is beyond specified offset 0x{:x}", index, number));
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
                    let instvariant : InstVariant = ron::from_str(&varstr).map_err(|err| err.to_string())?;
                    let cmp : CmpFlags = cmpstr.try_into().map_err(|_| "invalid CmpFlags")?;
                    let sign = match signstr {
                        "+" => false,
                        "-" => true,
                        _ => { return Err(String::from("Invalid sign")); }
                    };
                    let instr = Instruction { cmp: cmp, sign: sign, var: instvariant };
                    let opcode : u32 = instr.into();
                    println!("{:08x}  {}  {:x}", index, instr, opcode);
                    outfile.write_all(&opcode.to_le_bytes()).unwrap();
                    index += 4;
                }
                (Some(cmpstr), Some(signstr), Some(varstr)) if cmpstr.is_empty() && signstr.is_empty() && varstr.is_empty() => {}
                (None, None, None) => {}
                _ => { return Err(String::from("Invalid line")); }
            }

            let immediatestr = split.next();
            match immediatestr {
                Some(str) if (!str.is_empty()) => {
                    let number : u32 = u32::from_str_radix(str.trim_start_matches("0x"), 16).map_err(|err| err.to_string())?;
                    outfile.write_all(&number.to_le_bytes()).unwrap();
                    index += 4;
                }
                _ => {}
            };
            Ok(())
        })() {
            Ok(()) => {}
            Err(err) => { panic!("Error parsing line {}\nText: {}\nError message: {}", linum, l, err); }
        }
    }

}
