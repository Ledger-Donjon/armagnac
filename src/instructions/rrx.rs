//! Implements RRX (Rotate Right with Extend) instruction.

use super::Encoding::{self, T1};
use super::{
    ArmVersion::{V7EM, V7M},
    Pattern,
};
use crate::arm::Effect;
use crate::{
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// RRX instruction.
///
/// Rotate Right with Extend.
pub struct Rrx {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for Rrx {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM],
            expression: "11101010010x1111(0)000xxxx0011xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rm,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let (result, carry) = shift_c(proc[self.rm], Shift::rrx(), carry_in);
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "rrx".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::{rrx::Rrx, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_rrx() {
        struct Test {
            set_flags: bool,
            carry_in: bool,
            initial_rm: u32,
            expected_rd: u32,
            expected_flags: u8,
        }

        let vectors = [
            Test {
                set_flags: true,
                initial_rm: 4,
                carry_in: true,
                expected_rd: 0x80000002,
                expected_flags: 0b10000,
            },
            Test {
                set_flags: true,
                initial_rm: 1,
                carry_in: false,
                expected_rd: 0x00000000,
                expected_flags: 0b01100,
            },
            Test {
                set_flags: false,
                initial_rm: 1,
                carry_in: false,
                expected_rd: 0x00000000,
                expected_flags: 0,
            },
            Test {
                set_flags: true,
                initial_rm: 0x87654321,
                carry_in: true,
                expected_rd: 0xc3b2a190,
                expected_flags: 0b10100,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v7m());
            let rd = RegisterIndex::new_general_random();
            let rm = RegisterIndex::new_general_random();
            proc.set(rm, v.initial_rm);
            proc.registers.psr.set_c(v.carry_in);
            let mut expected = proc.registers.clone();
            expected.psr.set_flags(v.expected_flags);
            expected.set(rd, v.expected_rd);
            Rrx {
                rd,
                rm,
                set_flags: v.set_flags,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
