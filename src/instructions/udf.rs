//! Implements UDF (Undefined) instruction.

use super::ArmVersion::{V6M, V7EM, V7M, V8M};
use super::{DecodeHelper, Instruction, Pattern};
use crate::arm::{ArmProcessor, RunError};
use crate::decoder::DecodeError;
use crate::it_state::ItState;

/// UDF (Undefined) instruction.
pub struct Udf {
    /// Immediate constant stored by the instruction.
    /// This constant is ignored by the processor.
    imm16: u16,
}

impl Instruction for Udf {
    fn patterns() -> &'static [super::Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "11011110xxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "111101111111xxxx1010xxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                imm16: ins.imm8(0) as u16,
            },
            2 => Self {
                imm16: ((ins.imm4(16) << 12) | ins.imm12(0)) as u16,
            },
            _ => panic!(),
        })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<bool, RunError> {
        return Err(RunError::InstructionUndefined);
    }

    fn name(&self) -> String {
        "udf".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("#{}", self.imm16)
    }
}

#[cfg(test)]
mod tests {
    use super::Udf;
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M, RunError},
        instructions::Instruction,
    };

    #[test]
    fn test_udf() {
        let mut proc = ArmProcessor::new(V7M, 0);
        let result = Udf { imm16: 0 }.execute(&mut proc);
        assert_eq!(result, Err(RunError::InstructionUndefined));
    }
}
