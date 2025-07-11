//! Implements REVSH (Byte-Reverse Signed Halfword) instruction.

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

/// REVSH instruction.
///
/// Byte-Reverse Signed Halfword.
pub struct Revsh {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rm: RegisterIndex,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Revsh {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011101011xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110101001xxxx1111xxxx1011xxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        match encoding {
            T1 => Ok(Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                encoding,
            }),
            T2 => {
                let rd = ins.reg4(8);
                let rm1 = ins.reg4(0);
                let rm2 = ins.reg4(16);
                unpredictable(rm1 != rm2)?;
                unpredictable(rd.is_sp_or_pc() || rm1.is_sp_or_pc())?;
                Ok(Self {
                    rd,
                    rm: rm1,
                    encoding,
                })
            }
            _ => panic!(),
        }
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rm = proc[self.rm];
        let result = (((((rm as u8) as i8) as i32) as u32) << 8) | ((rm & 0x0000ff00) >> 8);
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "revsh".into()
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
    use super::Revsh;
    use crate::{
        core::{Config, Processor},
        instructions::{Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_revsh() {
        let vectors = [(0x12345678, 0x00007856), (0x12b456f8, 0xfffff856)];
        for v in vectors {
            let mut proc = Processor::new(Config::v7m());
            let (rd, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rm, v.0);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.1);
            Revsh {
                rd,
                rm,
                encoding: DontCare,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
