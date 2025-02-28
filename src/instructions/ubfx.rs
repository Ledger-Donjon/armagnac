//! Implements UBFX (Unsigned Bit Field Extract) instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper, ItState},
    registers::RegisterIndex,
};

use super::Instruction;

pub struct Ubfx {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Least significant bit number.
    lsb: u8,
    /// Bitfield width.
    width_minus_1: u8,
}

impl Instruction for Ubfx {
    fn patterns() -> &'static [&'static str] {
        &["11110(0)111100xxxx0xxxxxxxxx(0)xxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let lsb = ((ins >> 12 & 7) << 2 | ins >> 6 & 3) as u8;
        let width_minus_1 = (ins & 0x1f) as u8;
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
        unpredictable(lsb + width_minus_1 > 31)?;
        Ok(Self {
            rd,
            rn,
            lsb: ((ins >> 12 & 7) << 2 | ins >> 6 & 3) as u8,
            width_minus_1: (ins & 0x1f) as u8,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let msb = self.lsb + self.width_minus_1;
        debug_assert!(msb <= 31);
        let result = proc.registers[self.rn] << (31 - msb) >> (31 - msb + self.lsb);
        proc.registers.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "ubfx".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, #{}, #{}",
            self.rd,
            self.rn,
            self.lsb,
            self.width_minus_1 + 1
        )
    }
}
