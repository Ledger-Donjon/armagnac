//! Implements BFC (Bit Field Clear) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::core::{Processor, Effect, RunError};
use crate::{
    core::ItState,
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// BFC instruction.
///
/// Bit Field Clear.
pub struct Bfc {
    /// Destination register.
    rd: RegisterIndex,
    /// LSB to be cleared.
    /// Ranges from 0 to 31 included.
    lsb: u8,
    /// MSB to be cleared.
    /// Ranges from `lsb` to 31 included.
    msb: u8,
}

impl Instruction for Bfc {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110(0)11011011110xxxxxxxxx(0)xxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        unpredictable(rd.is_sp_or_pc())?;
        Ok(Self {
            rd,
            lsb: ((ins.imm3(12) << 2) | ins.imm2(6)) as u8,
            msb: (ins.imm5(0)) as u8,
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        if self.msb >= self.lsb {
            let width = self.msb - self.lsb + 1;
            let mask = !((0xffffffffu32 >> (32 - width)) << self.lsb);
            let value = proc[self.rd] & mask;
            proc.set(self.rd, value);
        } else {
            return Err(RunError::Unpredictable);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "bfc".into()
    }

    fn args(&self, _pc: u32) -> String {
        let width = self.msb - self.lsb + 1;
        format!("{}, #{}, #{}", self.rd, self.lsb, width)
    }
}

#[cfg(test)]
mod tests {
    use super::Bfc;
    use crate::{
        core::{Processor, Config, RunError},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    #[test]
    fn test_bfc() {
        let vectors = [
            (0, 0, 0b11111111_11111111_11111111_11111110),
            (0, 2, 0b11111111_11111111_11111111_11111000),
            (0, 29, 0b11000000_00000000_00000000_00000000),
            (0, 30, 0b10000000_00000000_00000000_00000000),
            (0, 31, 0b00000000_00000000_00000000_00000000),
            (4, 7, 0b11111111_11111111_11111111_00001111),
            (8, 15, 0b11111111_11111111_00000000_11111111),
            (16, 23, 0b11111111_00000000_11111111_11111111),
            (24, 30, 0b10000000_11111111_11111111_11111111),
            (24, 31, 0b00000000_11111111_11111111_11111111),
        ];

        for v in vectors {
            let mut proc = Processor::new(Config::v8m());
            let rd = RegisterIndex::new_general_random();
            proc.set(rd, 0xffffffff);
            Bfc {
                rd,
                lsb: v.0,
                msb: v.1,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc[rd], v.2);
        }

        // Check that msb < lsb leads to error.
        let mut proc = Processor::new(Config::v8m());
        let rd = RegisterIndex::new_general_random();
        assert_eq!(
            Bfc {
                rd,
                lsb: 10,
                msb: 9
            }
            .execute(&mut proc),
            Err(RunError::Unpredictable)
        );
    }
}
