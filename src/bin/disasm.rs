
use std::{fs::File, io::Read, ops::Range};

use alea_core::prelude::*;

use bytes::BytesMut;
use clap::Parser;


fn parse_range(s: &str) -> Result<Range::<u32>, &'static str> {
    let (x, y) = s.split_once('-').ok_or("missing -")?;

    let x_fromstr = u32::from_str_radix(x, 16).map_err(|_| "ParseError")?;
    let y_fromstr = u32::from_str_radix(y, 16).map_err(|_| "ParseError")?;

    Ok(Range::<u32> { start: x_fromstr, end: y_fromstr })
}

fn is_in_range(x: u32, ranges: &Vec<Range::<u32>>) -> bool {
    for range in ranges {
        if range.contains(&x) {
            return true
        }
    }
    return false
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug)]
struct Cli {
    /// Skip blocks of at least this number of null bytes
    #[arg(short = 'z', long, default_value_t = 12)]
    skip_zeros: u32,
    /// Force interpretation as data blocks
    #[arg(short = 'd', long, value_parser = parse_range)]
    force_data: Vec<Range::<u32>>,
    /// The path to the file to read
    infile: std::path::PathBuf,
}

fn main() {
    let cli = Cli::parse();
    println!("# {:?}", cli);

	let mut buffer = Vec::new();
	let mut file = File::open(cli.infile).unwrap();

	file.read_to_end(&mut buffer).unwrap();

    let program = BytesMut::from(buffer.as_slice());

    let mut index: u32 = 0;

    while index < program.size() {
        let instr_word = program.read_u32(index);
        if instr_word == 0 && cli.skip_zeros > 0 {
            let mut zeroidx = index;
            while zeroidx < program.size() && program.read_u32(zeroidx + 4) == 0 {
                zeroidx += 4;
            }
            if zeroidx - index > cli.skip_zeros {
                index = zeroidx + 4;
                print!("\n\n{:04x}: ", index);
                continue;
            }
        }
        if is_in_range(index, &cli.force_data) {
            println!("\t\t\t\t0x{:08x}", instr_word);
            index += 4;
            continue;
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
