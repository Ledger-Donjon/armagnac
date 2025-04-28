//! Implements ADR (Address to Register) instruction.

use super::ArmVersion::{V6M, V7M, V8M};
use super::{unpredictable, DecodeHelper, Instruction, Pattern, Qualifier};
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
    tn: usize,
}

impl Instruction for Adr {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "10100xxxxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "11110x10101011110xxxxxxxxxxxxxxx",
            },
            Pattern {
                tn: 3,
                versions: &[V7M, V8M],
                expression: "11110x10000011110xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(
        tn: usize,
        ins: u32,
        _state: crate::it_state::ItState,
    ) -> Result<Self, crate::decoder::DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(8),
                imm32: (ins.imm8(0) as i32) << 2,
                tn,
            },
            2 | 3 => {
                let rd = ins.reg4(8);
                unpredictable(rd.is_sp_or_pc())?;
                let imm12 = ((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0)) as i32;
                Self {
                    rd,
                    imm32: if tn == 2 { -imm12 } else { imm12 },
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::ArmProcessor) -> Result<bool, crate::arm::RunError> {
        let result = proc.pc().align(4).wrapping_add(self.imm32 as u32);
        proc.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "adr".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 2 | 3)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rd, self.imm32)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::ArmProcessor,
        instructions::{adr::Adr, Instruction},
        registers::RegisterIndex,
    };

    fn test_adr_vec(proc: &mut ArmProcessor, offset: i32) {
        proc.set_pc(0x1000);
        proc.registers.r0 = 0;
        let rd = RegisterIndex::new_general_random();
        Adr {
            rd,
            imm32: offset,
            tn: 0,
        }
        .execute(proc)
        .unwrap();
        assert_eq!(proc[rd], (0x1000 as i32 + offset) as u32);
    }

    #[test]
    fn test_adr() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        test_adr_vec(&mut proc, 0);
        test_adr_vec(&mut proc, -16);
        test_adr_vec(&mut proc, 16);
    }
}
