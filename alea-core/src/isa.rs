/// xgayuh instruction set
/// fuck it we ball
///
///
/// 32 bit instruction width
/// all instructions optional via state flag comparison
/// mmapped i/o
/// 16 registers
/// 32bit address space
///
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    CS,
    CA,
    ST,
    IA,
    RA,
    SA,
}

const ALL_REGISTERS: [Register; 16] = [
    Register::R0,
    Register::R1,
    Register::R2,
    Register::R3,
    Register::R4,
    Register::R5,
    Register::R6,
    Register::R7,
    Register::R8,
    Register::R9,
    Register::CS,
    Register::CA,
    Register::ST,
    Register::IA,
    Register::RA,
    Register::SA,
];

impl TryFrom<u8> for Register {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Register::*;

        match value {
            0b0000 => Ok(R0),
            0b0001 => Ok(R1),
            0b0010 => Ok(R2),
            0b0011 => Ok(R3),
            0b0100 => Ok(R4),
            0b0101 => Ok(R5),
            0b0110 => Ok(R6),
            0b0111 => Ok(R7),
            0b1000 => Ok(R8),
            0b1001 => Ok(R9),
            0b1010 => Ok(CS),
            0b1011 => Ok(CA),
            0b1100 => Ok(ST),
            0b1101 => Ok(IA),
            0b1110 => Ok(RA),
            0b1111 => Ok(SA),
            _ => Err(()),
        }
    }
}

impl Into<u8> for Register {
    fn into(self) -> u8 {
        use Register::*;

        match self {
            R0 => 0b0000,
            R1 => 0b0001,
            R2 => 0b0010,
            R3 => 0b0011,
            R4 => 0b0100,
            R5 => 0b0101,
            R6 => 0b0110,
            R7 => 0b0111,
            R8 => 0b1000,
            R9 => 0b1001,
            CS => 0b1010,
            CA => 0b1011,
            ST => 0b1100,
            IA => 0b1101,
            RA => 0b1110,
            SA => 0b1111,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct State(pub u32);

bitflags::bitflags! {
    impl State: u32 {
        const CMP_GT = 1;
        const CMP_EQ = 1 << 1;
        const CMP_LT = 1 << 2;
        const CMP_EN = 1 << 3;
        const RET_ST = 1 << 4;
    }
}

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct CmpFlags {
    pub gt: bool,
    pub eq: bool,
    pub lt: bool,
}

impl CmpFlags {
    pub fn new<T: Ord>(a: T, b: T) -> Self {
        CmpFlags {
            gt: a > b,
            eq: a == b,
            lt: a < b,
        }
    }
}

impl Display for CmpFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CmpFlags({})",
            match (self.gt, self.eq, self.lt) {
                (true, true, true) => "ALL",
                (true, true, false) => "GE",
                (true, false, true) => "NE",
                (false, true, true) => "LE",
                (true, false, false) => "GT",
                (false, true, false) => "EQ",
                (false, false, true) => "LT",
                (false, false, false) => "NONE",
            },
        )
    }
}

impl Into<&str> for CmpFlags {
    fn into(self) -> &'static str {
        match (self.gt, self.eq, self.lt) {
            (true, true, true) => "ALL",
            (true, true, false) => "GE",
            (true, false, true) => "NE",
            (false, true, true) => "LE",
            (true, false, false) => "GT",
            (false, true, false) => "EQ",
            (false, false, true) => "LT",
            (false, false, false) => "NONE",
        }
    }
}

impl TryFrom<&str> for CmpFlags {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ALL" => Ok(CmpFlags { gt: true, eq: true, lt: true }),
            "GE" => Ok(CmpFlags { gt: true, eq: true, lt: false }),
            "NE" => Ok(CmpFlags { gt: true, eq: false, lt: true }),
            "LE" => Ok(CmpFlags { gt: false, eq: true, lt: true }),
            "GT" => Ok(CmpFlags { gt: true, eq: false, lt: false }),
            "EQ" => Ok(CmpFlags { gt: false, eq: true, lt: false }),
            "LT" => Ok(CmpFlags { gt: false, eq: false, lt: true }),
            "NONE" => Ok(CmpFlags { gt: false, eq: false, lt: false }),
            &_ => Err(()),
        }
    }
}

impl Into<State> for CmpFlags {
    fn into(self) -> State {
        self.gt.then_some(State::CMP_GT).unwrap_or(State(0))
            | self.eq.then_some(State::CMP_EQ).unwrap_or(State(0))
            | self.lt.then_some(State::CMP_LT).unwrap_or(State(0))
    }
}

impl Default for State {
    fn default() -> Self {
        Self::CMP_EQ | Self::CMP_GT | Self::CMP_LT
    }
}

impl State {
    pub fn set_cmp(&mut self, new: CmpFlags) {
        self.set(Self::CMP_GT, new.gt);
        self.set(Self::CMP_EQ, new.eq);
        self.set(Self::CMP_LT, new.lt);
    }

    pub fn get_cmp(&self) -> CmpFlags {
        CmpFlags {
            gt: self.contains(Self::CMP_GT),
            eq: self.contains(Self::CMP_EQ),
            lt: self.contains(Self::CMP_LT),
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct RegisterFile {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    /// call stack
    pub cs: u32,
    /// "carry", used by the alu for extended results
    pub ca: u32,
    /// "state", internal state and alu flags
    pub st: State,
    /// instruction address
    pub ia: u32,
    /// return address
    pub ra: u32,
    /// stack address
    pub sa: u32,
}

impl Index<u8> for RegisterFile {
    type Output = u32;

    fn index(&self, index: u8) -> &Self::Output {
        assert!(index < 16, "there's only 16 registers");
        match index {
            0 => &self.r0,
            1 => &self.r1,
            2 => &self.r2,
            3 => &self.r3,
            4 => &self.r4,
            5 => &self.r5,
            6 => &self.r6,
            7 => &self.r7,
            8 => &self.r8,
            9 => &self.r9,
            10 => &self.cs,
            11 => &self.ca,
            12 => &self.st.0,
            13 => &self.ia,
            14 => &self.ra,
            15 => &self.sa,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<u8> for RegisterFile {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        assert!(index < 16, "there's only 16 registers");
        match index {
            0 => &mut self.r0,
            1 => &mut self.r1,
            2 => &mut self.r2,
            3 => &mut self.r3,
            4 => &mut self.r4,
            5 => &mut self.r5,
            6 => &mut self.r6,
            7 => &mut self.r7,
            8 => &mut self.r8,
            9 => &mut self.r9,
            10 => &mut self.cs,
            11 => &mut self.ca,
            12 => &mut self.st.0,
            13 => &mut self.ia,
            14 => &mut self.ra,
            15 => &mut self.sa,
            _ => unreachable!(),
        }
    }
}

impl Index<Register> for RegisterFile {
    type Output = u32;

    fn index(&self, index: Register) -> &Self::Output {
        use Register::*;

        match index {
            R0 => &self.r0,
            R1 => &self.r1,
            R2 => &self.r2,
            R3 => &self.r3,
            R4 => &self.r4,
            R5 => &self.r5,
            R6 => &self.r6,
            R7 => &self.r7,
            R8 => &self.r8,
            R9 => &self.r9,
            CS => &self.cs,
            CA => &self.ca,
            ST => &self.st.0,
            IA => &self.ia,
            RA => &self.ra,
            SA => &self.sa,
        }
    }
}

impl IndexMut<Register> for RegisterFile {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        use Register::*;

        match index {
            R0 => &mut self.r0,
            R1 => &mut self.r1,
            R2 => &mut self.r2,
            R3 => &mut self.r3,
            R4 => &mut self.r4,
            R5 => &mut self.r5,
            R6 => &mut self.r6,
            R7 => &mut self.r7,
            R8 => &mut self.r8,
            R9 => &mut self.r9,
            CS => &mut self.cs,
            CA => &mut self.ca,
            ST => &mut self.st.0,
            IA => &mut self.ia,
            RA => &mut self.ra,
            SA => &mut self.sa,
        }
    }
}

impl Display for RegisterFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        out += "RegisterFile {\n";
        for regs in ALL_REGISTERS.chunks(4) {
            for r in regs {
                out += format!("  {:?}: {:#010x}", r, self[*r]).as_str();
            }
            out += "\n";
        }
        out += "};";
        write!(f, "{}", out)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Instruction {
    pub cmp: CmpFlags,
    pub sign: bool,
    pub var: InstVariant,
}

impl Instruction {
    pub fn has_upcoming_value(&self) -> bool {
        use InstVariant::*;
        use InstructionData::*;
        match self.var {
            // this might be the worst match i've ever written
            Set | Rst | Ret | Nop | Halt => false,
            Add(id) | Mul(id) | And(id) | Or(id) | Xor(id) | Not(id) | Cmp(id) | Lsl(id)
            | Lsr(id) | Asr(id) | Ror(id) | Ld(id) | Sto(id) | Push(id) | Pop(id) | Mov(id)
            | Call(id) | Jmp(id) | JmpAbs(id) => match id {
                RegisterAndU32(_, _) | TwoRegistersAndU32(_, _, _) | ImmediateOnly(_) => true,
                _ => false,
            },
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Instruction({} {} {:?})",
            self.cmp,
            if self.sign { "Neg" } else { "Pos" },
            self.var
        )
    }
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
/// first 5 bits: instruction variant
/// last 16 bits: decode into instruction data
/// bits 6-8: cmp bits, gt eq lt
/// 9-12: 4 bits of cozy padding
/// 13: sign bit
/// 14-16: instruction data variant
pub enum InstVariant {
    // data processing
    Add(InstructionData),
    Mul(InstructionData),
    And(InstructionData),
    Or(InstructionData),
    Xor(InstructionData),
    Not(InstructionData),

    // comparison, branch control
    Cmp(InstructionData),
    Set,
    Rst,

    // shifts
    Lsl(InstructionData),
    Lsr(InstructionData),
    Asr(InstructionData),
    Ror(InstructionData),

    // Load and store and move
    Ld(InstructionData),
    Sto(InstructionData),
    Push(InstructionData),
    Pop(InstructionData),
    Mov(InstructionData),

    // subroutine and jumps
    Call(InstructionData),
    Ret,
    Jmp(InstructionData),
    JmpAbs(InstructionData),

    // misc
    #[default]
    Nop,
    Halt,
}

/// a marker to make the cpu read the next 4 bytes from the program code as a literal value
pub type UpcomingU32 = ();

// up to 16 bits large
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum InstructionData {
    OneRegister(Register),
    TwoRegisters(Register, Register),
    ThreeRegisters(Register, Register, Register),
    RegisterAndU8(Register, u8),
    RegisterAndU32(Register, UpcomingU32),
    TwoRegistersAndU8(Register, Register, u8),
    TwoRegistersAndU32(Register, Register, UpcomingU32),
    ImmediateOnly(UpcomingU32),
}
