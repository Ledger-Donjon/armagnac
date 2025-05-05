//! Implements USAT16 (Unsigned Saturate 16) instruction.

use crate::{
    arith::unsigned_sat_q,
    arm::{
        ArmProcessor,
        ArmVersion::{V7M, V8M},
        RunError,
    },
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{Instruction, Pattern};

/// USAT16 instruction.
///
/// Unsigned Saturate 16.
pub struct Usat16 {
    /// Destination register.
    rd: RegisterIndex,
    /// Bit position for saturation, in range 0 to 15.
    saturate_to: u8,
    /// Register containing the value to be saturated.
    rn: RegisterIndex,
}

impl Instruction for Usat16 {
    fn patterns() -> &'static [super::Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V8M],
            expression: "11110(0)111010xxxx0000xxxx00(0)(0)xxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
        Ok(Self {
            rd,
            saturate_to: ins.imm4(0) as u8,
            rn,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let (result1, sat1) = unsigned_sat_q(rn as i16 as i64, self.saturate_to);
        let (result2, sat2) = unsigned_sat_q((rn >> 16) as u16 as i16 as i64, self.saturate_to);
        proc.set(
            self.rd,
            (result1 as u16 as u32) | ((result2 as u16 as u32) << 16),
        );
        if sat1 || sat2 {
            proc.registers.psr.set_q(true);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "usat16".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}, {}", self.rd, self.saturate_to, self.rn)
    }
}

#[cfg(test)]
mod tests {
    use super::Usat16;
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    #[test]
    fn test_usat16() {
        let vectors = [
            (0x00000000, 6, 0x00000000, false),
            (0x003f003f, 6, 0x003f003f, false),
            (0x0040003f, 6, 0x003f003f, true),
            (0x003f0040, 6, 0x003f003f, true),
            (0x7fff7fff, 16, 0x7fff7fff, false),
            (0xf1230123, 16, 0x00000123, true),
            (0x7890f432, 16, 0x78900000, true),
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            let rd = RegisterIndex::new_general_random();
            let rn = RegisterIndex::new_general_random();
            proc.set(rn, v.0);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.2);
            expected.psr.set_q(v.3);
            Usat16 {
                rd,
                saturate_to: v.1,
                rn,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
