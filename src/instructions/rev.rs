//! Implements REV (Byte-Reverse Word) instruction.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{unpredictable, DecodeHelper, Instruction};

/// REV instruction.
pub struct Rev {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rm: RegisterIndex,
}

impl Instruction for Rev {
    fn patterns() -> &'static [&'static str] {
        &["1011101000xxxxxx", "111110101001xxxx1111xxxx1000xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
            },
            2 => {
                let rm1 = ins.reg4(0);
                let rm2 = ins.reg4(16);
                let rd = ins.reg4(8);
                unpredictable(rm1 != rm2)?;
                unpredictable(rd.is_sp_or_pc() || rm1.is_sp_or_pc())?;
                Self { rd, rm: rm1 }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rm = proc.registers[self.rm];
        let result = (rm & 0xff) << 24 | (rm & 0xff00) << 8 | (rm & 0xff0000) >> 8 | rm >> 24;
        proc.registers.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "rev".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::ArmProcessor,
        instructions::{rev::Rev, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_rev() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 0x12345678;
        let ins = Rev {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x78563412);
    }
}
