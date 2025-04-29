//! Implements RSB (Reverse Subtract) instruction.

use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    it_state::ItState,
    registers::RegisterIndex,
};

/// RSB (immediate) instruction.
pub struct RsbImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand value.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    tn: usize,
}

impl Instruction for RsbImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "0100001001xxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "11110x01110xxxxx0xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: 0,
                set_flags: !state.in_it_block(),
                tn,
            },
            2 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
                let imm12 = (((ins >> 26) & 1) << 11) | (ins.imm3(12) << 8) | ins.imm8(0);
                Self {
                    rd,
                    rn,
                    imm32: thumb_expand_imm(imm12)?,
                    set_flags: ins.bit(20),
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let (result, carry, overflow) = add_with_carry(!proc[self.rn], self.imm32, true);
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers
                .psr
                .set_nz(result)
                .set_c(carry)
                .set_v(overflow);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "rsb".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 2)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rn, self.imm32)
    }
}

pub struct RsbReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift applied to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for RsbReg {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V8M],
            expression: "11101011110xxxxx(0)xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rn,
            rm,
            shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let (shifted, _) = shift_c(proc[self.rm], self.shift, carry_in);
        let rn = proc[self.rn];
        let (result, carry, overflow) = add_with_carry(!rn, shifted, true);
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers
                .psr
                .set_nz(result)
                .set_c(carry)
                .set_v(overflow);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "rsb".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}{}",
            self.rd,
            self.rn,
            self.rm,
            self.shift.arg_string()
        )
    }
}
