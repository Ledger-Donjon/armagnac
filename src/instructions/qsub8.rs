//! Implements QSUB8 (Saturating Subtract 8) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// QSUB8 instruction.
///
/// Saturating Subtract 8.
pub struct Qsub8 {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Qsub8 {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7EM, V8M],
            expression: "111110101100xxxx1111xxxx0001xxxx",
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
        let diff1 = (rn as i8).saturating_sub(rm as i8);
        let diff2 = ((rn >> 8) as i8).saturating_sub((rm >> 8) as i8);
        let diff3 = ((rn >> 16) as i8).saturating_sub((rm >> 16) as i8);
        let diff4 = ((rn >> 24) as i8).saturating_sub((rm >> 24) as i8);
        let result = ((diff4 as u32) << 24)
            | (((diff3 as u8) as u32) << 16)
            | (((diff2 as u8) as u32) << 8)
            | (diff1 as u8) as u32;
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "qsub8".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{Processor, Config},
        instructions::{qsub8::Qsub8, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qsub8() {
        struct Test {
            initial_rn: u32,
            initial_rm: u32,
            expected_rd: u32,
        }

        let vectors = [
            Test {
                initial_rn: 0x81818181,
                initial_rm: 0x01010101,
                expected_rd: 0x80808080,
            },
            Test {
                initial_rn: 0x81818181,
                initial_rm: 0x02020202,
                expected_rd: 0x80808080,
            },
            Test {
                initial_rn: 0x7d7d7d7d,
                initial_rm: 0xfefefefe,
                expected_rd: 0x7f7f7f7f,
            },
            Test {
                initial_rn: 0x7e7e7e7e,
                initial_rm: 0xfefefefe,
                expected_rd: 0x7f7f7f7f,
            },
            Test {
                initial_rn: 0x12342143,
                initial_rm: 0x11223344,
                expected_rd: 0x0112eeff,
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
            Qsub8 { rd, rm, rn }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
