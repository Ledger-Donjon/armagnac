//! Implements REV (Byte-Reverse Word) instruction.

use super::Encoding::{self, T1, T2};
use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    registers::RegisterIndex,
};

/// REV instruction.
pub struct Rev {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rm: RegisterIndex,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Rev {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011101000xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110101001xxxx1111xxxx1000xxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                encoding,
            },
            T2 => {
                let rm1 = ins.reg4(0);
                let rm2 = ins.reg4(16);
                let rd = ins.reg4(8);
                unpredictable(rm1 != rm2)?;
                unpredictable(rd.is_sp_or_pc() || rm1.is_sp_or_pc())?;
                Self {
                    rd,
                    rm: rm1,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rm = proc[self.rm];
        let result =
            ((rm & 0xff) << 24) | ((rm & 0xff00) << 8) | ((rm & 0xff0000) >> 8) | (rm >> 24);
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "rev".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{Config, Processor},
        instructions::{rev::Rev, Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_rev() {
        let mut proc = Processor::new(Config::v8m());
        proc.registers.r1 = 0x12345678;
        let ins = Rev {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
            encoding: DontCare,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x78563412);
    }
}
