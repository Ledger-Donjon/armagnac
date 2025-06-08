//! Implements SADD16 (Signed Add 16) instruction.

use super::ArmVersion::{V7EM, V8M};
use super::Encoding::{self, T1};
use super::{Instruction, Pattern};
use crate::{
    core::ItState,
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// SADD16 instruction.
///
/// Signed Add 16.
pub struct Sadd16 {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Sadd16 {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7EM, V8M],
            expression: "111110101001xxxx1111xxxx0000xxxx",
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

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let rm = proc[self.rm];
        let rn = proc[self.rn];
        let sum1 = (rn as i16 as i32).wrapping_add(rm as i16 as i32);
        let sum2 = ((rn >> 16) as i16 as i32).wrapping_add((rm >> 16) as i16 as i32);
        proc.set(
            self.rd,
            ((sum1 as u16) as u32) | (((sum2 as u16) as u32) << 16),
        );
        let ge10 = if sum1 >= 0 { 0b0011 } else { 0 };
        let ge32 = if sum2 >= 0 { 0b1100 } else { 0 };
        proc.registers.psr.set_ge(ge10 | ge32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "sadd16".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use super::Sadd16;
    use crate::core::{ArmProcessor, Config};
    use crate::instructions::Instruction;
    use crate::registers::RegisterIndex;

    #[test]
    fn test_sadd16() {
        let vectors = [
            (0x46620505, 0x12370505, 0x58990a0a, 0b1111),
            (0x7fff07d0, 0x4320f448, 0xc31ffc18, 0b1100),
            (0xdb007f00, 0x024155aa, 0xdd41d4aa, 0b0011),
            (0x12347ffe, 0x80008000, 0x9234fffe, 0b0000),
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v7em());
            let rd = RegisterIndex::new_general_random();
            let (rn, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rn, v.0);
            proc.set(rm, v.1);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.2);
            expected.psr.set_ge(v.3);
            Sadd16 { rd, rn, rm }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected)
        }
    }
}
