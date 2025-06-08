//! Implements UDF (Undefined) instruction.

use super::ArmVersion::{V6M, V7EM, V7M, V8M};
use super::Encoding::{self, T1, T2};
use super::{DecodeHelper, Instruction, Pattern};
use crate::core::ItState;
use crate::core::{ArmProcessor, Effect, RunError};
use crate::decoder::DecodeError;

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
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "11011110xxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "111101111111xxxx1010xxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                imm16: ins.imm8(0) as u16,
            },
            T2 => Self {
                imm16: ((ins.imm4(16) << 12) | ins.imm12(0)) as u16,
            },
            _ => panic!(),
        })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        Err(RunError::InstructionUndefined)
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
        core::{ArmProcessor, Config, RunError},
        instructions::Instruction,
    };

    #[test]
    fn test_udf() {
        let mut proc = ArmProcessor::new(Config::v7m());
        let result = Udf { imm16: 0 }.execute(&mut proc);
        assert_eq!(result, Err(RunError::InstructionUndefined));
    }
}
