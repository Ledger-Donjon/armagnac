//! Implements UMULL (Unsigned Multiply Long) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// UMULL instruction.
///
/// Unsigned Multiply Long.
pub struct Umull {
    /// Lower 32 bits of the result.
    rdlo: RegisterIndex,
    /// Upper 32 bits of the result.
    rdhi: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Umull {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110111010xxxxxxxxxxxx0000xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rdlo = ins.reg4(12);
        let rdhi = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(
            rdlo.is_sp_or_pc() | rdhi.is_sp_or_pc() | rn.is_sp_or_pc() | rm.is_sp_or_pc(),
        )?;
        unpredictable(rdhi == rdlo)?;
        Ok(Self { rdlo, rdhi, rn, rm })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let result = proc[self.rn] as u64 * proc[self.rm] as u64;
        proc.set(self.rdhi, (result >> 32) as u32);
        proc.set(self.rdlo, result as u32);
        Ok(false)
    }

    fn name(&self) -> String {
        "umull".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}, {}", self.rdlo, self.rdhi, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::{umull::Umull, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_umull() {
        let mut proc = ArmProcessor::new(Config::v8m());
        proc.registers.r2 = 0x12345678;
        proc.registers.r3 = 0x87654321;
        let ins = Umull {
            rdlo: RegisterIndex::R0,
            rdhi: RegisterIndex::R1,
            rn: RegisterIndex::R2,
            rm: RegisterIndex::R3,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x70b88d78);
        assert_eq!(proc.registers.r1, 0x09a0cd05);

        proc.registers.r2 = 0x11223344;
        proc.registers.r3 = 0xff00ff00;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xcd10bc00);
        assert_eq!(proc.registers.r1, 0x11112221);
    }
}
