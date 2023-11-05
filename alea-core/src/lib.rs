mod decode;
pub mod isa;
pub mod memory;
pub mod vcore;

pub mod prelude {
    pub use crate::isa::{
        CmpFlags, InstVariant, Instruction, InstructionData, Register, RegisterFile, State,
    };
    pub use crate::memory::{MappedMemory, Memory, MemoryMap};
    pub use crate::vcore::{CPUError, CPU};
}
