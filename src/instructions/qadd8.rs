//! Implements QADD8 (Saturating Add 8) instruction.

use super::Instruction;
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{rdn_args_string, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// QADD8 instruction.
///
/// Saturating Add 8.
pub struct Qadd8 {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Qadd8 {
    fn patterns() -> &'static [&'static str] {
        &["111110101000xxxx1111xxxx0001xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rn, rm })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn];
        let rm = proc.registers[self.rm];
        let sum1 = (rn as i8).saturating_add(rm as i8);
        let sum2 = ((rn >> 8) as i8).saturating_add((rm >> 8) as i8);
        let sum3 = ((rn >> 16) as i8).saturating_add((rm >> 16) as i8);
        let sum4 = ((rn >> 24) as i8).saturating_add((rm >> 24) as i8);
        let result = ((sum4 as u32) << 24)
            | (((sum3 as u8) as u32) << 16)
            | (((sum2 as u8) as u32) << 8)
            | (sum1 as u8) as u32;
        proc.registers.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "qadd8".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rn), self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::{qadd8::Qadd8, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qadd8() {
        struct Test {
            rd: RegisterIndex,
            rm: RegisterIndex,
            rn: RegisterIndex,
            initial_rm: u32,
            initial_rn: u32,
            expected_rd: u32,
        }

        let vectors = [
            Test {
                rd: RegisterIndex::R0,
                rm: RegisterIndex::R1,
                rn: RegisterIndex::R2,
                initial_rm: 0x7e7e7e7e,
                initial_rn: 0x01010101,
                expected_rd: 0x7f7f7f7f,
            },
            Test {
                rd: RegisterIndex::R1,
                rm: RegisterIndex::R2,
                rn: RegisterIndex::R3,
                initial_rm: 0x7e7e7e7e,
                initial_rn: 0x02020202,
                expected_rd: 0x7f7f7f7f,
            },
            Test {
                rd: RegisterIndex::R2,
                rm: RegisterIndex::R3,
                rn: RegisterIndex::R4,
                initial_rm: 0x81818181,
                initial_rn: 0xffffffff,
                expected_rd: 0x80808080,
            },
            Test {
                rd: RegisterIndex::R3,
                rm: RegisterIndex::R4,
                rn: RegisterIndex::R5,
                initial_rm: 0x81818181,
                initial_rn: 0xf0f0f0f0,
                expected_rd: 0x80808080,
            },
            Test {
                rd: RegisterIndex::R4,
                rm: RegisterIndex::R5,
                rn: RegisterIndex::R6,
                initial_rm: 0x11223344,
                initial_rn: 0x12342143,
                expected_rd: 0x2356547f,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            proc.registers.set(v.rm, v.initial_rm);
            proc.registers.set(v.rn, v.initial_rn);
            let mut expected = proc.registers.clone();
            expected.set(v.rd, v.expected_rd);
            Qadd8 {
                rd: v.rd,
                rm: v.rm,
                rn: v.rn,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
