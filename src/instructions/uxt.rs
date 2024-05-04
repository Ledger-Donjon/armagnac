//! Implements UXTB and UXTH instructions.

use crate::{
    arith::ror,
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{unpredictable, DecodeHelper, Instruction};

/// UXTB instruction.
pub struct Uxtb {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Rotation applied to Rm.
    rotation: u8,
}

impl Instruction for Uxtb {
    fn patterns() -> &'static [&'static str] {
        &["1011001011xxxxxx", "11111010010111111111xxxx1(0)xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                rotation: 0,
            },
            2 => {
                let rd = ins.reg4(8);
                let rm = ins.reg4(0);
                unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rm,
                    rotation: (ins.imm2(4) << 3) as u8,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rotated = ror(proc[self.rm], self.rotation as u32);
        proc[self.rd] = rotated;
        Ok(false)
    }

    fn name(&self) -> String {
        "uxtb".into()
    }

    fn args(&self, _pc: u32) -> String {
        let ror_str = if self.rotation != 0 {
            format!(", ROR #{}", self.rotation)
        } else {
            "".into()
        };
        format!("{}, {}{}", self.rd, self.rm, ror_str)
    }
}
