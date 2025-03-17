//! Implements CMP (Compare) instruction.

use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{unpredictable, DecodeHelper, Instruction};

/// CMP (immediate) instruction.
pub struct CmpImm {
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand immediate value.
    imm32: u32,
}

impl Instruction for CmpImm {
    fn patterns() -> &'static [&'static str] {
        &["00101xxxxxxxxxxx", "11110x011011xxxx0xxx1111xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rn: ins.reg3(8),
                imm32: ins & 0xff,
            },
            2 => Self {
                rn: ins.reg4(16),
                imm32: thumb_expand_imm((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff)?,
            },
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let (result, carry, overflow) = add_with_carry(proc[self.rn], !self.imm32, true);
        proc.registers
            .psr
            .set_nz(result)
            .set_c(carry)
            .set_v(overflow);
        Ok(false)
    }

    fn name(&self) -> String {
        "cmp".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rn, self.imm32)
    }
}

/// CMP (register) instruction.
pub struct CmpReg {
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to apply to Rm.
    shift: Shift,
}

impl Instruction for CmpReg {
    fn patterns() -> &'static [&'static str] {
        &[
            "0100001010xxxxxx",
            "01000101xxxxxxxx",
            "111010111011xxxx(0)xxx1111xxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rn: ins.reg3(0),
                rm: ins.reg3(3),
                shift: Shift::lsl(0),
            },
            2 => {
                let rn = (ins.imm1(7) << 3) | ins.imm3(0);
                let rm = ins.imm4(3);
                unpredictable(rn < 8 && rm < 8)?;
                let rn = RegisterIndex::new_main(rn);
                let rm = RegisterIndex::new_main(rm);
                unpredictable(rn.is_pc() || rm.is_pc())?;
                Self {
                    rn,
                    rm,
                    shift: Shift::lsl(0),
                }
            }
            3 => {
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                unpredictable(rn.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let shifted = shift_c(proc[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc[self.rn], !shifted, true);
        proc.registers
            .psr
            .set_nz(result)
            .set_c(carry)
            .set_v(overflow);
        Ok(false)
    }

    fn name(&self) -> String {
        "cmp".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}{}", self.rn, self.rm, self.shift.arg_string())
    }
}
