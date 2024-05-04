//! Implements MVN (immediate) and MVN (register) instructions.

use crate::{
    arith::thumb_expand_imm_optc,
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::DecodeHelper,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// MVN (immediate) instruction.
pub struct MvnImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Immediate value
    imm32: u32,
    /// Carry
    carry: Option<bool>,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for MvnImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x00011x11110xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let imm12 = ins.imm1(26) << 11 | ins.imm3(12) << 8 | ins.imm8(0);
        let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
        Ok(Self {
            rd: ins.reg4(8),
            imm32,
            carry,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let result = !self.imm32;
        proc[self.rd] = result;
        if self.set_flags {
            proc.registers.apsr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "mvns" } else { "mvn" }.into()
    }

    fn args(&self, pc: u32) -> String {
        format!("{}, #{}", self.rd, self.imm32)
    }
}
