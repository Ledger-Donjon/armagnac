//! Implements SMLAL (Signed Multiply Accumulate Long) instruction.

use super::{
    Encoding::{self, T1},
    Instruction, Pattern,
};
use crate::{
    core::ItState,
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// SMLAL instruction.
///
/// Signed Multiply Accumulate Long.
pub struct Smlal {
    /// Source register for the lower 32 bits of the accumulate value, and destination register for
    /// the lower 32 bits of the result.
    rdlo: RegisterIndex,
    /// Source register for the higher 32 bits of the accumulate value, and destination register
    /// for the higher 32 bits of the result.
    rdhi: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Smlal {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110111100xxxxxxxxxxxx0000xxxx",
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

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let result = (proc[self.rn] as i32 as i64) * (proc[self.rm] as i32 as i64)
            + (((proc[self.rdhi] as u64) << 32) | proc[self.rdlo] as u64) as i64;
        proc.set(self.rdhi, (result >> 32) as u32);
        proc.set(self.rdlo, result as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "smlal".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}, {}", self.rdlo, self.rdhi, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use super::Smlal;
    use crate::{
        core::{Config, Processor},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    #[test]
    fn test_smlal() {
        let vectors: [(i64, i32, i32, i64); 5] = [
            (0, 10, 10, 100),
            (1000, 36, -41, -476),
            (
                -0x1234123412341234,
                0x10101010,
                0x78789696,
                -767012380396832980,
            ),
            (
                2264415127291284243,
                482331322,
                1684641743,
                3076970606288858489,
            ),
            (
                8898870332397265231,
                -2127066525,
                109506214,
                8665943330318378881,
            ),
        ];

        for v in vectors {
            let mut proc = Processor::new(Config::v7m());
            let (rdlo, rdhi, rn, rm) = RegisterIndex::pick_four_general_distinct();
            proc.set(rdhi, (v.0 >> 32) as u32);
            proc.set(rdlo, v.0 as u32);
            proc.set(rn, v.1 as u32);
            proc.set(rm, v.2 as u32);
            let mut expected = proc.registers.clone();
            expected.set(rdhi, (v.3 >> 32) as u32);
            expected.set(rdlo, v.3 as u32);
            Smlal { rdlo, rdhi, rn, rm }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected)
        }
    }
}
