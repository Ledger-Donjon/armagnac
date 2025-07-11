//! Implements QADD8 (Saturating Add 8) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
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
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7EM, V8M],
            expression: "111110101000xxxx1111xxxx0001xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rn, rm })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rn = proc[self.rn];
        let rm = proc[self.rm];
        let sum1 = (rn as i8).saturating_add(rm as i8);
        let sum2 = ((rn >> 8) as i8).saturating_add((rm >> 8) as i8);
        let sum3 = ((rn >> 16) as i8).saturating_add((rm >> 16) as i8);
        let sum4 = ((rn >> 24) as i8).saturating_add((rm >> 24) as i8);
        let result = ((sum4 as u32) << 24)
            | (((sum3 as u8) as u32) << 16)
            | (((sum2 as u8) as u32) << 8)
            | (sum1 as u8) as u32;
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "qadd8".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{Config, Processor},
        instructions::{qadd8::Qadd8, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qadd8() {
        struct Test {
            initial_rm: u32,
            initial_rn: u32,
            expected_rd: u32,
        }

        let vectors = [
            Test {
                initial_rm: 0x7e7e7e7e,
                initial_rn: 0x01010101,
                expected_rd: 0x7f7f7f7f,
            },
            Test {
                initial_rm: 0x7e7e7e7e,
                initial_rn: 0x02020202,
                expected_rd: 0x7f7f7f7f,
            },
            Test {
                initial_rm: 0x81818181,
                initial_rn: 0xffffffff,
                expected_rd: 0x80808080,
            },
            Test {
                initial_rm: 0x81818181,
                initial_rn: 0xf0f0f0f0,
                expected_rd: 0x80808080,
            },
            Test {
                initial_rm: 0x11223344,
                initial_rn: 0x12342143,
                expected_rd: 0x2356547f,
            },
        ];

        for v in vectors {
            let mut proc = Processor::new(Config::v7em());
            let rd = RegisterIndex::new_general_random();
            let (rm, rn) = RegisterIndex::pick_two_general_distinct();
            proc.set(rm, v.initial_rm);
            proc.set(rn, v.initial_rn);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_rd);
            Qadd8 { rd, rm, rn }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
