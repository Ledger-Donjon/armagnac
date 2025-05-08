//! Implements SMULL (Signed Multiply Long) instruction.

use super::{
    Encoding::{self, T1},
    Instruction, Pattern,
};
use crate::{arm::Effect, registers::RegisterIndex};
use crate::{
    arm::{
        ArmProcessor,
        ArmVersion::{V7EM, V7M, V8M},
        RunError,
    },
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
};

/// SMULL instruction.
///
/// Signed Multiply Long.
pub struct Smull {
    /// Destination register for the lower 32 bits of the result.
    rdlo: RegisterIndex,
    /// Destination register for the higher 32 bits of the result.
    rdhi: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Smull {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110111000xxxxxxxxxxxx0000xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rdlo = ins.reg4(12);
        let rdhi = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(
            rdlo.is_sp_or_pc() || rdhi.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc(),
        )?;
        unpredictable(rdlo == rdhi)?;
        Ok(Self { rdlo, rdhi, rn, rm })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let result = (proc[self.rn] as i32 as i64) * (proc[self.rm] as i32 as i64);
        proc.set(self.rdhi, (result >> 32) as u32);
        proc.set(self.rdlo, result as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "smull".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}, {}", self.rdlo, self.rdhi, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use super::Smull;
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    #[test]
    fn test_smlal() {
        let vectors: [(i32, i32, i64); 5] = [
            (10, 10, 100),
            (36, -41, -1476),
            (0x10101010, 0x78789696, 544681025927825760),
            (-482331322, 1684641743, -812555478997574246),
            (2127066525, -109506214, -232927002078886350),
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v7m());
            let (rdlo, rdhi, rn, rm) = RegisterIndex::pick_four_general_distinct();
            proc.set(rn, v.0 as u32);
            proc.set(rm, v.1 as u32);
            let mut expected = proc.registers.clone();
            expected.set(rdhi, (v.2 >> 32) as u32);
            expected.set(rdlo, v.2 as u32);
            Smull { rdlo, rdhi, rn, rm }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected)
        }
    }
}
