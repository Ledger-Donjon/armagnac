//! Implements RRX (Rotate Right with Extend) instruction.

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
    fn patterns() -> &'static [&'static str] {
        &["11101010010x1111(0)000xxxx0011xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rm,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (result, carry) = shift_c(proc.registers[self.rm], Shift::rrx(), carry_in);
        proc.registers.set(self.rd, result);
        if self.set_flags {
            println!("DEBUG {:?}", carry);
            proc.registers.xpsr.set_nz(result).set_c(carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "rrxs" } else { "rrx" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::{rrx::Rrx, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_rrx() {
        struct Test {
            rd: RegisterIndex,
            rm: RegisterIndex,
            set_flags: bool,
            carry_in: bool,
            initial_rm: u32,
            expected_rd: u32,
            expected_nzcv: (bool, bool, bool, bool),
        }

        let vectors = [
            Test {
                rd: RegisterIndex::R0,
                rm: RegisterIndex::R1,
                set_flags: true,
                initial_rm: 4,
                carry_in: true,
                expected_rd: 0x80000002,
                expected_nzcv: (true, false, false, false),
            },
            Test {
                rd: RegisterIndex::R1,
                rm: RegisterIndex::R2,
                set_flags: true,
                initial_rm: 1,
                carry_in: false,
                expected_rd: 0x00000000,
                expected_nzcv: (false, true, true, false),
            },
            Test {
                rd: RegisterIndex::R1,
                rm: RegisterIndex::R2,
                set_flags: false,
                initial_rm: 1,
                carry_in: false,
                expected_rd: 0x00000000,
                expected_nzcv: (false, false, false, false),
            },
            Test {
                rd: RegisterIndex::R1,
                rm: RegisterIndex::R2,
                set_flags: true,
                initial_rm: 0x87654321,
                carry_in: true,
                expected_rd: 0xc3b2a190,
                expected_nzcv: (true, false, true, false),
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            proc.registers.set(v.rm, v.initial_rm);
            proc.registers.xpsr.set_c(v.carry_in);
            let mut expected = proc.registers.clone();
            expected.xpsr.set_n(v.expected_nzcv.0);
            expected.xpsr.set_z(v.expected_nzcv.1);
            expected.xpsr.set_c(v.expected_nzcv.2);
            expected.xpsr.set_v(v.expected_nzcv.3);
            expected.set(v.rd, v.expected_rd);
            Rrx {
                rd: v.rd,
                rm: v.rm,
                set_flags: v.set_flags,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
