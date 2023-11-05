use std::io;

use tracing::{debug, info, trace};

use crate::{
    isa::{CmpFlags, InstVariant, Instruction, InstructionData, RegisterFile, State},
    memory::{Memory, MemoryMap},
};

#[derive(Default)]
pub struct CPU {
    pub mmap: MemoryMap,
    pub reg: RegisterFile,

    pub step_wait: bool,
}

#[derive(Debug)]
pub enum CPUError {
    InvalidInstruction,
    Unimplemented,
    Halt,
}

fn wait() {
    io::stdin()
        .read_line(&mut String::new())
        .expect("couldn't read stdin");
}

impl CPU {
    pub fn run_until_halt(&mut self) -> Result<(), CPUError> {
        loop {
            match self.step() {
                Ok(_) => continue,
                Err(CPUError::Halt) => {
                    info!("halting");
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn call_peripherals_pre(&mut self) {
        self.mmap.mems.iter_mut().for_each(|m| m.inner.pre_step())
    }

    pub fn call_peripherals_post(&mut self) {
        self.mmap.mems.iter_mut().for_each(|m| m.inner.post_step())
    }

    pub fn step(&mut self) -> Result<(), CPUError> {
        use InstVariant::*;
        use InstructionData::*;

        debug!("calling peripherals pre-step");
        self.call_peripherals_pre();

        let current_ia = self.reg.ia;
        let current_inst: Instruction = self.read_ia().try_into()?;

        debug!("running instruction at {:#010x}", current_ia);
        trace!("register dump:\n{}", self.reg);

        if self.step_wait {
            info!("will run {}", current_inst);
            info!("waiting...");
            wait();
        }

        if self.reg.st.contains(State::CMP_EN)
            && (self.reg.st & current_inst.cmp.into()) == State(0)
        {
            debug!("optional instruction skipped!");
            if current_inst.has_upcoming_value() {
                self.read_ia();
            }
            return Ok(());
        }

        match current_inst.var {
            Nop => (),
            Halt => return Err(CPUError::Halt),
            Add(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    RegisterAndU32(r, _) => (r, r, self.read_ia()),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    TwoRegistersAndU32(t, a, _) => (t, a, self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = if current_inst.sign {
                    self.reg[a] - b
                } else {
                    self.reg[a] + b
                };
            }
            And(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    RegisterAndU32(r, _) => (r, r, self.read_ia()),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    TwoRegistersAndU32(t, a, _) => (t, a, self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = self.reg[a] & b;
            }
            Or(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    RegisterAndU32(r, _) => (r, r, self.read_ia()),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    TwoRegistersAndU32(t, a, _) => (t, a, self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = self.reg[a] | b;
            }
            Xor(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    RegisterAndU32(r, _) => (r, r, self.read_ia()),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    TwoRegistersAndU32(t, a, _) => (t, a, self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = self.reg[a] ^ b;
            }
            Not(id) => {
                let (t, v) = match id {
                    OneRegister(r) => (r, self.reg[r]),
                    TwoRegisters(t, v) => (t, self.reg[v]),
                    RegisterAndU8(t, v) => (t, v as u32),
                    RegisterAndU32(t, _) => (t, self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = !v;
            }
            Mul(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    RegisterAndU32(r, _) => (r, r, self.read_ia()),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    TwoRegistersAndU32(t, a, _) => (t, a, self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = self.reg[a] * b;
            }
            Cmp(id) => {
                let (a, b) = match id {
                    OneRegister(a) => (a, 0),
                    TwoRegisters(a, b) => (a, self.reg[b]),
                    RegisterAndU8(a, b) => (a, b as u32),
                    RegisterAndU32(a, _) => (a, self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg.st.set_cmp(CmpFlags::new(self.reg[a], b));
            }
            Set => {
                self.reg.st.set(State::CMP_EN, current_inst.sign);
            }
            Rst => self.reg.st.set_cmp(CmpFlags {
                gt: true,
                eq: true,
                lt: true,
            }),
            Lsl(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = self.reg[a] << b;
            }
            Lsr(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = self.reg[a] >> b;
            }
            Asr(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                // rust does asr for signed ints, and lsr for unsigned
                self.reg[t] = ((self.reg[a] as i32) >> b) as u32;
            }
            Ror(id) => {
                let (t, a, b) = match id {
                    TwoRegisters(a, b) => (a, a, self.reg[b]),
                    ThreeRegisters(t, a, b) => (t, a, self.reg[b]),
                    RegisterAndU8(r, v) => (r, r, v as u32),
                    TwoRegistersAndU8(t, a, b) => (t, a, b as u32),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[t] = self.reg[a].rotate_right(b);
            }
            Ld(id) => {
                let (a, r) = match id {
                    TwoRegisters(r, a) => (self.reg[a], r),
                    RegisterAndU32(r, _) => (self.read_ia(), r),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[r] = self.mmap.read_u32(a);
            }
            Sto(id) => {
                let (v, a) = match id {
                    TwoRegisters(v, a) => (self.reg[v], self.reg[a]),
                    RegisterAndU32(v, _) => (self.reg[v], self.read_ia()),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.mmap.write_u32(a, v);
            }
            Push(id) => {
                match id {
                    OneRegister(a) => vec![self.reg[a]],
                    TwoRegisters(a, b) => vec![self.reg[a], self.reg[b]],
                    ThreeRegisters(a, b, c) => vec![self.reg[a], self.reg[b], self.reg[c]],
                    RegisterAndU8(a, b) => vec![self.reg[a], b as u32],
                    RegisterAndU32(a, _) => vec![self.reg[a], self.read_ia()],
                    TwoRegistersAndU8(a, b, c) => vec![self.reg[a], self.reg[b], c as u32],
                    TwoRegistersAndU32(a, b, _) => vec![self.reg[a], self.reg[b], self.read_ia()],
                    ImmediateOnly(_) => vec![self.read_ia()],
                }
                .into_iter()
                .for_each(|val| self.push_stack(val));
            }
            Pop(id) => {
                match id {
                    OneRegister(a) => vec![a],
                    TwoRegisters(a, b) => vec![a, b],
                    ThreeRegisters(a, b, c) => vec![a, b, c],
                    _ => return Err(CPUError::InvalidInstruction),
                }
                .into_iter()
                .for_each(|r| self.reg[r] = self.pop_stack());
            }
            Mov(id) => {
                let (r, v) = match id {
                    TwoRegisters(a, b) => (a, self.reg[b]),
                    RegisterAndU32(a, _) => (a, self.read_ia()),
                    RegisterAndU8(a, v) => (a, v as u32),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg[r] = v;
            }
            Call(id) => {
                let call_addr = match id {
                    OneRegister(r) => self.reg[r],
                    ImmediateOnly(_) => self.read_ia(),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.push_cs();
                self.reg.ra = self.reg.ia;
                self.reg.ia = call_addr;
            }
            Ret => {
                self.reg.ia = self.reg.ra;
                self.pop_cs();
            }
            Jmp(id) => {
                let jump_offset = match id {
                    OneRegister(r) => self.reg[r],
                    ImmediateOnly(_) => self.read_ia(),
                    RegisterAndU8(_, v) | TwoRegistersAndU8(_, _, v) => (v as u32) * 4,
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg.ia = if current_inst.sign {
                    current_ia - jump_offset
                } else {
                    current_ia + jump_offset
                };
            }
            JmpAbs(id) => {
                let jump_addr = match id {
                    OneRegister(r) => self.reg[r],
                    ImmediateOnly(_) => self.read_ia(),
                    _ => return Err(CPUError::InvalidInstruction),
                };
                self.reg.ia = jump_addr;
            }
        };

        debug!("calling peripherals post-step");
        self.call_peripherals_post();

        Ok(())
    }

    fn read_ia(&mut self) -> u32 {
        let v = self.mmap.read_u32(self.reg.ia);
        self.reg.ia += 4;
        trace!("read_ia: {:#010x}", v);
        v
    }

    fn push_stack(&mut self, v: u32) {
        self.reg.sa -= 4;
        debug!("pushed {:#010x} to {:#010x}", v, self.reg.sa);
        self.mmap.write_u32(self.reg.sa, v);
    }

    fn pop_stack(&mut self) -> u32 {
        let v = self.mmap.read_u32(self.reg.sa);
        debug!("popped {:#010x} from {:#010x}", v, self.reg.sa);
        self.reg.sa += 4;
        v
    }

    fn push_cs(&mut self) {
        self.reg.cs -= 4;
        self.mmap.write_u32(self.reg.cs, self.reg.ra);
        debug!("pushed {:#010x} onto callstack", self.reg.ra);
    }

    fn pop_cs(&mut self) {
        self.reg.ra = self.mmap.read_u32(self.reg.cs);
        self.reg.cs += 4;
        debug!("popped {:#010x} from callstack", self.reg.ra);
    }
}
