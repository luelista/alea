use crate::{
    isa::{CmpFlags, InstVariant, Instruction, InstructionData},
};

impl Into<u32> for CmpFlags {
    fn into(self) -> u32 {
        (if self.gt { 0x04000000 } else { 0 }) | (if self.eq { 0x02000000 } else { 0 }) | (if self.lt { 0x01000000 } else { 0 })
    }
}



fn makeinstr(opcode: u32, cmpcode: u32, sign: bool, idcode: u32) -> u32 {
    (opcode << 27) | cmpcode | (if sign { 0x00080000 } else { 0 }) | idcode
}

impl Into<u32> for Instruction {
    fn into(self) -> u32 {
        use InstVariant::*;

        match self.var {
            Nop => makeinstr(0b00000, self.cmp.into(), self.sign, 0),
            Halt => makeinstr(0b00000, self.cmp.into(), self.sign, 1),
            Cmp(id) => makeinstr(0b00001, self.cmp.into(), self.sign, id.into()),
            Set => makeinstr(0b00010, self.cmp.into(), self.sign, 0),
            Rst => makeinstr(0b00011, self.cmp.into(), self.sign, 0),
            Call(id) => makeinstr(0b00100, self.cmp.into(), self.sign, id.into()),
            Ret => makeinstr(0b00101, self.cmp.into(), self.sign, 0),
            Jmp(id) => makeinstr(0b00110, self.cmp.into(), self.sign, id.into()),
            JmpAbs(id) => makeinstr(0b00111, self.cmp.into(), self.sign, id.into()),
            Ld(id) => makeinstr(0b01000, self.cmp.into(), self.sign, id.into()),
            Sto(id) => makeinstr(0b01001, self.cmp.into(), self.sign, id.into()),
            Push(id) => makeinstr(0b01011, self.cmp.into(), self.sign, id.into()),
            Pop(id) => makeinstr(0b01100, self.cmp.into(), self.sign, id.into()),
            Mov(id) => makeinstr(0b01101, self.cmp.into(), self.sign, id.into()),
            Add(id) => makeinstr(0b10000, self.cmp.into(), self.sign, id.into()),
            Mul(id) => makeinstr(0b10001, self.cmp.into(), self.sign, id.into()),
            And(id) => makeinstr(0b10010, self.cmp.into(), self.sign, id.into()),
            Or(id) => makeinstr(0b10011, self.cmp.into(), self.sign, id.into()),
            Xor(id) => makeinstr(0b10100, self.cmp.into(), self.sign, id.into()),
            Not(id) => makeinstr(0b10101, self.cmp.into(), self.sign, id.into()),
            Lsl(id) => makeinstr(0b11000, self.cmp.into(), self.sign, id.into()),
            Lsr(id) => makeinstr(0b11001, self.cmp.into(), self.sign, id.into()),
            Asr(id) => makeinstr(0b11010, self.cmp.into(), self.sign, id.into()),
            Ror(id) => makeinstr(0b11011, self.cmp.into(), self.sign, id.into()),
        }
    }
}


fn makedata(variant: u32, reg1: u8, reg2: u8, reg3: u8, v: u8) -> u32 {
    variant << 16 | u32::from(reg1) << 12 | u32::from(reg2) << 8 | u32::from(reg3) << 4 | u32::from(v)
}

impl Into<u32> for InstructionData {
    fn into(self) -> u32 {
        use InstructionData::*;
        
        match self {
            RegisterAndU8(r1, v) => makedata(0b000, r1.into(), 0, 0, v),
            RegisterAndU32(r1, _v) => makedata(0b001, r1.into(), 0, 0, 0),
            TwoRegistersAndU8(r1, r2, v) => makedata(0b010, r1.into(), r2.into(), 0, v),
            TwoRegistersAndU32(r1, r2, _v) => makedata(0b011, r1.into(), r2.into(), 0, 0),
            OneRegister(r1) => makedata(0b100, r1.into(), 0, 0, 0),
            TwoRegisters(r1, r2) => makedata(0b101, r1.into(), r2.into(), 0, 0),
            ThreeRegisters(r1, r2, r3) => makedata(0b110, r1.into(), r2.into(), r3.into(), 0),
            ImmediateOnly(_v) => makedata(0b111, 0, 0, 0, 0),
        }
    }
}
