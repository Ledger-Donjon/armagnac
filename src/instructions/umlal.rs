//! Implements UMLAL (Unsigned Multiply Accumulate Long) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// UMLAL instruction.
///
/// Unsigned Multiply Accumulate Long.
pub struct Umlal {
    /// Lower 32 bits of the accumulate value and destination register for the result lower 32
    /// bits.
    rdlo: RegisterIndex,
    /// Upper 32 bits of the accumulate value and destination register for the result upper 32
    /// bits.
    rdhi: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Umlal {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V8M],
            expression: "111110111110xxxxxxxxxxxx0000xxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
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
        let result = (proc[self.rn] as u64 * proc[self.rm] as u64)
            .wrapping_add(((proc[self.rdhi] as u64) << 32) | proc[self.rdlo] as u64);
        proc.set(self.rdhi, (result >> 32) as u32);
        proc.set(self.rdlo, result as u32);
        Ok(false)
    }

    fn name(&self) -> String {
        "umlal".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}, {}", self.rdlo, self.rdhi, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::ArmProcessor,
        instructions::{umlal::Umlal, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_umlal() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r2 = 0x12345678;
        proc.registers.r3 = 0x87654321;
        let ins = Umlal {
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
        assert_eq!(proc.registers.r0, 0x3dc94978);
        assert_eq!(proc.registers.r1, 0x1ab1ef27);
    }
}
