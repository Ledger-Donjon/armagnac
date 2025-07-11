//! Implements MOVT (Move Top) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// MOVT instruction.
///
/// Move Top.
pub struct Movt {
    /// Destination register.
    rd: RegisterIndex,
    /// Immediate value to be written to Rd.
    imm16: u16,
}

impl Instruction for Movt {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110x101100xxxx0xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        unpredictable(rd.is_sp_or_pc())?;
        let imm16 =
            ((ins.imm4(16) << 12) | (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0))
                as u16;
        Ok(Self { rd, imm16 })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rd = proc[self.rd];
        proc.set(self.rd, ((self.imm16 as u32) << 16) | rd & 0x0000ffff);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "movt".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rd, self.imm16)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{Config, Processor},
        instructions::{movt::Movt, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_movt() {
        let vectors = [
            (RegisterIndex::R0, 0x1234, 0x12344321),
            (RegisterIndex::R1, 0, 0x4321),
            (RegisterIndex::R2, 0xffff, 0xffff4321),
        ];
        for v in vectors {
            let mut proc = Processor::new(Config::v7m());
            proc.set(v.0, 0x87654321);
            let mut expected = proc.registers.clone();
            Movt {
                rd: v.0,
                imm16: v.1,
            }
            .execute(&mut proc)
            .unwrap();
            expected.set(v.0, v.2);
            assert_eq!(proc.registers, expected);
        }
    }
}
