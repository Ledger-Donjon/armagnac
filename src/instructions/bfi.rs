//! Implements BFI (Bit Field Insert) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{other, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// BFI instruction.
///
/// Bit Field Insert.
pub struct Bfi {
    /// Destination register.
    rd: RegisterIndex,
    /// Source register.
    rn: RegisterIndex,
    /// Least significant destination bit index.
    /// Ranges from 0 to 31 included.
    lsb: u8,
    /// Most significant destination bit index.
    /// Ranges from `lsb` to 31 included.
    msb: u8,
}

impl Instruction for Bfi {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110(0)110110xxxx0xxxxxxxxx(0)xxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // BFC
        unpredictable(rd.is_sp_or_pc() || rn.is_sp())?;
        Ok(Self {
            rd,
            rn,
            lsb: ((ins.imm3(12) << 2) | ins.imm2(6)) as u8,
            msb: ins.imm5(0) as u8,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        if self.msb >= self.lsb {
            let width = self.msb - self.lsb + 1;
            let mask = 0xffffffffu32 >> (32 - width);
            let value = (proc.registers[self.rd] & !(mask << self.lsb))
                | ((proc.registers[self.rn] & mask) << self.lsb);
            proc.registers.set(self.rd, value);
        } else {
            return Err(RunError::Unpredictable);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "bfi".into()
    }

    fn args(&self, _pc: u32) -> String {
        let width = self.msb - self.lsb + 1;
        format!("{}, {}, #{}, #{}", self.rd, self.rn, self.lsb, width)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, Config, RunError},
        instructions::{bfi::Bfi, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_bfi() {
        let vectors = [
            (0, 0, 0x12b456f9),
            (0, 7, 0x12b45621),
            (8, 15, 0x12b421f8),
            (16, 31, 0x432156f8),
            (0, 31, 0x87654321),
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v8m());
            let (rd, rn) = RegisterIndex::pick_two_general_distinct();
            proc.set(rd, 0x12b456f8);
            proc.set(rn, 0x87654321);
            let mut expected_registers = proc.registers.clone();
            Bfi {
                rd,
                rn,
                lsb: v.0,
                msb: v.1,
            }
            .execute(&mut proc)
            .unwrap();
            expected_registers.set(rd, v.2);
            assert_eq!(proc.registers, expected_registers);
        }

        // Check that msb < lsb leads to error.
        let mut proc = ArmProcessor::new(Config::v8m());
        let rd = RegisterIndex::new_general_random();
        let rn = RegisterIndex::new_general_random();
        assert_eq!(
            Bfi {
                rd,
                rn,
                lsb: 10,
                msb: 9
            }
            .execute(&mut proc),
            Err(RunError::Unpredictable)
        );
    }
}
