use crate::{
    isa::{CmpFlags, InstVariant, Instruction, InstructionData},
    vcore::CPUError,
};

use tracing::{debug, trace};

impl TryFrom<u32> for Instruction {
    type Error = CPUError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use InstVariant::*;

        let cmp = value.into();
        let sign = (value & 0x00080000) != 0;

        let id = value.into();

        let inst = match value >> 27 {
            0b00000 => {
                if value > 0 {
                    // TODO: change this to only check low 16 bits, i.e. instdata
                    Halt
                } else {
                    Nop
                }
            }
            0b00001 => Cmp(id),
            0b00010 => Set,
            0b00011 => Rst,
            0b00100 => Call(id),
            0b00101 => Ret,
            0b00110 => Jmp(id),
            0b00111 => JmpAbs(id),
            0b01000 => Ld(id),
            0b01001 => Sto(id),
            0b01011 => Push(id),
            0b01100 => Pop(id),
            0b01101 => Mov(id),
            0b01110 => return Err(CPUError::InvalidInstruction),
            0b01111 => return Err(CPUError::InvalidInstruction),
            0b10000 => Add(id),
            0b10001 => Mul(id),
            0b10010 => And(id),
            0b10011 => Or(id),
            0b10100 => Xor(id),
            0b10101 => Not(id),
            0b10110 => return Err(CPUError::InvalidInstruction),
            0b10111 => return Err(CPUError::InvalidInstruction),
            0b11000 => Lsl(id),
            0b11001 => Lsr(id),
            0b11010 => Asr(id),
            0b11011 => Ror(id),
            0b11100 => return Err(CPUError::InvalidInstruction),
            0b11101 => return Err(CPUError::InvalidInstruction),
            0b11110 => return Err(CPUError::InvalidInstruction),
            0b11111 => return Err(CPUError::InvalidInstruction),

            _ => unreachable!(),
        };
        let i = Instruction {
            cmp,
            sign,
            var: inst,
        };

        debug!("decoded {:#010x} into {}", value, i);

        Ok(i)
    }
}

impl From<u32> for InstructionData {
    fn from(value: u32) -> Self {
        use InstructionData::*;
        let variant = (value & 0x00070000) >> 16;
        trace!("variant is {}, value is {:#010x}", variant, value);

        match variant {
            0b000 => {
                let r1 = ((value & 0x0000F000) >> 12) as u8;
                let v = (value & 0x000000FF) as u8;
                RegisterAndU8(r1.try_into().unwrap(), v)
            }
            0b001 => {
                let r1 = ((value & 0x0000F000) >> 12) as u8;
                RegisterAndU32(r1.try_into().unwrap(), ())
            }
            0b010 => {
                let r1 = ((value & 0x0000F000) >> 12) as u8;
                let r2 = ((value & 0x00000F00) >> 8) as u8;
                let v = (value & 0x000000FF) as u8;
                TwoRegistersAndU8(r1.try_into().unwrap(), r2.try_into().unwrap(), v)
            }
            0b011 => {
                let r1 = ((value & 0x0000F000) >> 12) as u8;
                let r2 = ((value & 0x00000F00) >> 8) as u8;
                TwoRegistersAndU32(r1.try_into().unwrap(), r2.try_into().unwrap(), ())
            }
            0b100 => {
                let r1 = ((value & 0x0000F000) >> 12) as u8;
                OneRegister(
                    r1.try_into()
                        .expect("this should only be 4 bits large hmmm"),
                )
            }
            0b101 => {
                let r1 = ((value & 0x0000F000) >> 12) as u8;
                let r2 = ((value & 0x00000F00) >> 8) as u8;
                TwoRegisters(r1.try_into().expect("fuck"), r2.try_into().expect("fuck"))
            }
            0b110 => {
                let r1 = ((value & 0x0000F000) >> 12) as u8;
                let r2 = ((value & 0x00000F00) >> 8) as u8;
                let r3 = ((value & 0x000000F0) >> 4) as u8;
                ThreeRegisters(
                    r1.try_into().unwrap(),
                    r2.try_into().unwrap(),
                    r3.try_into().unwrap(),
                )
            }
            0b111 => ImmediateOnly(()),
            _ => unreachable!(),
        }
    }
}

impl From<u32> for CmpFlags {
    fn from(value: u32) -> Self {
        let gt = (value & 0x04000000) != 0;
        let eq = (value & 0x02000000) != 0;
        let lt = (value & 0x01000000) != 0;
        CmpFlags { gt, eq, lt }
    }
}
