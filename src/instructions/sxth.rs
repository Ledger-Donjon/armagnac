//! Implements SXTH (Signed Extend Halfword) instruction.

use crate::{
    arith::ror,
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{unpredictable, DecodeHelper, Instruction};

/// SXTH instruction.
///
/// Unsigned Extend Halfword.
pub struct Sxth {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Rotation applied to Rm.
    /// Can be 0, 8, 16 or 24.
    rotation: u8,
}

impl Instruction for Sxth {
    fn patterns() -> &'static [&'static str] {
        &["1011001000xxxxxx", "11111010000011111111xxxx1(0)xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                rotation: 0,
            },
            2 => {
                let rm = ins.reg4(0);
                let rd = ins.reg4(8);
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

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rotated = ror(proc[self.rm], self.rotation as u32);
        proc.registers
            .set(self.rd, (((rotated & 0xffff) as i16) as i32) as u32);
        Ok(false)
    }

    fn name(&self) -> String {
        "sxth".into()
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

#[cfg(test)]
mod tests {
    use crate::{
        arm::ArmProcessor,
        instructions::{sxth::Sxth, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_sxth() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 0x12b456f8;

        let mut ins = Sxth {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
            rotation: 0,
        };

        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x56f8);

        ins.rotation = 8;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffb456);

        ins.rotation = 16;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x12b4);

        ins.rotation = 24;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffff812);
    }
}
