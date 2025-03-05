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
                imm32: thumb_expand_imm((ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff)?,
            },
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn];
        let (result, carry, overflow) = add_with_carry(rn, !self.imm32, true);
        proc.registers
            .xpsr
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
                let rn = (ins >> 7 & 1) << 3 | ins & 7;
                let rm = ins >> 3 & 0xf;
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
                    shift: Shift::from_bits(ins >> 4 & 3, (ins >> 12 & 7) << 2 | ins >> 6 & 3),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let shifted = shift_c(proc.registers[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc.registers[self.rn], !shifted, true);
        proc.registers
            .xpsr
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
