//! Implements ADR (Address to Register) instruction.

use super::{unpredictable, DecodeHelper, Instruction};
use crate::{align::Align, registers::RegisterIndex};

/// ADR instruction.
///
/// Address to Register.
pub struct Adr {
    /// Destination register.
    rd: RegisterIndex,
    /// Offset from PC.
    imm32: i32,
}

impl Instruction for Adr {
    fn patterns() -> &'static [&'static str] {
        &[
            "10100xxxxxxxxxxx",
            "11110x10101011110xxxxxxxxxxxxxxx",
            "11110x10000011110xxxxxxxxxxxxxxx",
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
            },
            2 | 3 => {
                let rd = ins.reg4(8);
                unpredictable(rd.is_sp_or_pc())?;
                let imm12 = ((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0)) as i32;
                Self {
                    rd,
                    imm32: if tn == 2 { -imm12 } else { imm12 },
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::ArmProcessor) -> Result<bool, crate::arm::RunError> {
        let result = proc.pc().align(4).wrapping_add(self.imm32 as u32);
        proc.registers.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "adr".into()
    }

    fn args(&self, pc: u32) -> String {
        let label = (pc as i32 + self.imm32) as u32 + 4;
        format!("{}, 0x{:x}", self.rd, label)
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
        }
        .execute(proc)
        .unwrap();
        assert_eq!(proc.registers[rd], (0x1000 as i32 + offset) as u32);
    }

    #[test]
    fn test_adr() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        test_adr_vec(&mut proc, 0);
        test_adr_vec(&mut proc, -16);
        test_adr_vec(&mut proc, 16);
    }
}
