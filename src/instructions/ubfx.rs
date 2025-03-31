//! Implements UBFX (Unsigned Bit Field Extract) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper, ItState},
    registers::RegisterIndex,
};

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
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V8M],
            expression: "11110(0)111100xxxx0xxxxxxxxx(0)xxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let lsb = ((ins.imm3(12) << 2) | ins.imm2(6)) as u8;
        let width_minus_1 = (ins & 0x1f) as u8;
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
        unpredictable(lsb + width_minus_1 > 31)?;
        Ok(Self {
            rd,
            rn,
            lsb,
            width_minus_1: ins.imm5(0) as u8,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let msb = self.lsb + self.width_minus_1;
        debug_assert!(msb <= 31);
        let result = proc[self.rn] << (31 - msb) >> (31 - msb + self.lsb);
        proc.set(self.rd, result);
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
