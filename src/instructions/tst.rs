//! Implements TST (Test) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// TST (immediate) instruction.
pub struct TstImm {
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand immediate value.
    imm32: u32,
    /// Carry.
    carry: Option<bool>,
}

impl Instruction for TstImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V8M],
            expression: "11110x000001xxxx0xxx1111xxxxxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rn = ins.reg4(16);
        unpredictable(rn.is_sp_or_pc())?;
        let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0);
        let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
        Ok(Self { rn, imm32, carry })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let result = proc[self.rn] & self.imm32;
        proc.registers.psr.set_nz(result).set_c_opt(self.carry);
        Ok(false)
    }

    fn name(&self) -> String {
        "tst".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rn, self.imm32)
    }
}

/// TST (register) instruction.
pub struct TstReg {
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
}

impl Instruction for TstReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "0100001000xxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "111010100001xxxx(0)xxx1111xxxxxxxx",
            },
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
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                unpredictable(rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
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
        let (shifted, carry) = shift_c(proc[self.rm], self.shift, carry_in);
        let result = proc[self.rn] & shifted;
        proc.registers.psr.set_nz(result).set_c(carry);
        Ok(false)
    }

    fn name(&self) -> String {
        "tst".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}{}", self.rn, self.rm, self.shift.arg_string())
    }
}
