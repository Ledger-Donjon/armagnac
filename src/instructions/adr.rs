//! Implements ADR (Address to Register) instruction.

use super::ArmVersion::{V6M, V7EM, V7M, V8M};
use super::Encoding::{self, T1, T2, T3};
use super::{unpredictable, DecodeHelper, Instruction, Pattern, Qualifier};
use crate::arm::{Effect, RunError};
use crate::qualifier_wide_match;
use crate::{align::Align, registers::RegisterIndex};

/// ADR instruction.
///
/// Address to Register.
pub struct Adr {
    /// Destination register.
    rd: RegisterIndex,
    /// Offset from PC.
    imm32: i32,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Adr {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "10100xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x10101011110xxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x10000011110xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(
        encoding: Encoding,
        ins: u32,
        _state: crate::it_state::ItState,
    ) -> Result<Self, crate::decoder::DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(8),
                imm32: (ins.imm8(0) as i32) << 2,
                encoding,
            },
            T2 | T3 => {
                let rd = ins.reg4(8);
                unpredictable(rd.is_sp_or_pc())?;
                let imm12 = ((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0)) as i32;
                Self {
                    rd,
                    imm32: if encoding == T2 { -imm12 } else { imm12 },
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::ArmProcessor) -> Result<Effect, RunError> {
        let result = proc.pc().align(4).wrapping_add(self.imm32 as u32);
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "adr".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2 | T3)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rd, self.imm32)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::{adr::Adr, Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    fn test_adr_vec(proc: &mut ArmProcessor, offset: i32) {
        proc.set_pc(0x1000);
        proc.registers.r0 = 0;
        let rd = RegisterIndex::new_general_random();
        Adr {
            rd,
            imm32: offset,
            encoding: DontCare,
        }
        .execute(proc)
        .unwrap();
        assert_eq!(proc[rd], (0x1000 as i32 + offset) as u32);
    }

    #[test]
    fn test_adr() {
        let mut proc = ArmProcessor::new(Config::v8m());
        test_adr_vec(&mut proc, 0);
        test_adr_vec(&mut proc, -16);
        test_adr_vec(&mut proc, 16);
    }
}
