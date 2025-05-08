//! Implements MLS (Multiply and Subtract) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::Effect;
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// MLS instruction.
///
/// Multiply and Subtract.
pub struct Mls {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Accumulator register.
    ra: RegisterIndex,
}

impl Instruction for Mls {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110110000xxxxxxxxxxxx0001xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        let ra = ins.reg4(12);
        unpredictable(
            rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc() || ra.is_sp_or_pc(),
        )?;
        Ok(Self { rd, rn, rm, ra })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let result = proc[self.ra] - proc[self.rn].wrapping_mul(proc[self.rm]);
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "mls".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}, {}", self.rd, self.rn, self.rm, self.ra)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::{mls::Mls, Instruction},
        registers::{CoreRegisters, RegisterIndex},
    };

    #[test]
    fn test_mls() {
        let mut proc = ArmProcessor::new(Config::v8m());
        proc.registers.r1 = 0x12345678;
        proc.registers.r2 = 0x01020304;
        proc.registers.r3 = 0x87654321;
        let expected_registers = CoreRegisters {
            r0: 0x7ca08141,
            ..proc.registers
        };
        let ins = Mls {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            ra: RegisterIndex::R3,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers, expected_registers);
    }
}
