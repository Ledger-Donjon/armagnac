//! Implements SADD8 (Signed Add 8) instruction.

use super::ArmVersion::{V7M, V8M};
use super::{Instruction, Pattern};
use crate::arm::{ArmProcessor, RunError};
use crate::decoder::DecodeError;
use crate::instructions::{unpredictable, DecodeHelper};
use crate::it_state::ItState;
use crate::registers::RegisterIndex;

/// SADD8 instruction.
///
/// Signed Add 8.
pub struct Sadd8 {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Sadd8 {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V8M],
            expression: "111110101000xxxx1111xxxx0000xxxx",
        }]
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
        let rm = proc[self.rm];
        let rn = proc[self.rn];
        let sum1 = (rn as i8 as i32).wrapping_add(rm as i8 as i32);
        let sum2 = ((rn >> 8) as i8 as i32).wrapping_add((rm >> 8) as i8 as i32);
        let sum3 = ((rn >> 16) as i8 as i32).wrapping_add((rm >> 16) as i8 as i32);
        let sum4 = ((rn >> 24) as i8 as i32).wrapping_add((rm >> 24) as i8 as i32);
        proc.set(
            self.rd,
            ((sum1 as u8) as u32)
                | ((sum2 as u8) as u32) << 8
                | ((sum3 as u8) as u32) << 16
                | ((sum4 as u8) as u32) << 24,
        );
        let ge0 = (sum1 >= 0) as u8;
        let ge1 = (sum2 >= 0) as u8;
        let ge2 = (sum3 >= 0) as u8;
        let ge3 = (sum4 >= 0) as u8;
        proc.registers
            .psr
            .set_ge(ge0 | (ge1 << 1) | (ge2 << 2) | (ge3 << 3));
        Ok(false)
    }

    fn name(&self) -> String {
        "sadd8".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::arm::ArmProcessor;
    use crate::arm::ArmVersion::V7M;
    use crate::instructions::Instruction;
    use crate::registers::RegisterIndex;

    use super::Sadd8;

    #[test]
    fn test_sadd8() {
        let vectors = [
            (0x05503f80, 0xfb162b7e, 0x00666afe, 0b1110),
            (0x007ff621, 0x0001f621, 0x0080ec42, 0b1101),
            (0x4225057f, 0xfe805001, 0x40a55580, 0b1011),
            (0xff7f1201, 0xff7feeff, 0xfefe0000, 0b0111),
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            let rd = RegisterIndex::new_general_random();
            let (rn, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rn, v.0);
            proc.set(rm, v.1);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.2);
            expected.psr.set_ge(v.3);
            Sadd8 { rd, rn, rm }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected)
        }
    }
}
