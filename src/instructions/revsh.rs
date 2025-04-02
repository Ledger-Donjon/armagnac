//! Implements REVSH (Byte-Reverse Signed Halfword) instruction.

use super::{unpredictable, DecodeHelper, Instruction};
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

/// REVSH instruction.
///
/// Byte-Reverse Signed Halfword.
pub struct Revsh {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rm: RegisterIndex,
}

impl Instruction for Revsh {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "1011101011xxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "111110101001xxxx1111xxxx1011xxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        match tn {
            1 => Ok(Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
            }),
            2 => {
                let rd = ins.reg4(8);
                let rm1 = ins.reg4(0);
                let rm2 = ins.reg4(16);
                unpredictable(rm1 != rm2)?;
                unpredictable(rd.is_sp_or_pc() || rm1.is_sp_or_pc())?;
                Ok(Self { rd, rm: rm1 })
            }
            _ => panic!(),
        }
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rm = proc[self.rm];
        let result = (((((rm as u8) as i8) as i32) as u32) << 8) | ((rm & 0x0000ff00) >> 8);
        proc.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "revsh".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use super::Revsh;
    use crate::{arm::ArmProcessor, instructions::Instruction, registers::RegisterIndex};

    #[test]
    fn test_revsh() {
        let vectors = [(0x12345678, 0x00007856), (0x12b456f8, 0xfffff856)];
        for v in vectors {
            let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V7M, 0);
            let (rd, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rm, v.0);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.1);
            Revsh { rd, rm }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
