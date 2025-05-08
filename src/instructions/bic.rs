//! Implements BIC (Bit Clear) instruction.

use super::Encoding::{self, T1, T2};
use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::Effect;
use crate::qualifier_wide_match;
use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::rdn_args_string,
    it_state::ItState,
    registers::RegisterIndex,
};

/// BIC (immediate) instruction.
pub struct BicImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand.
    imm32: u32,
    /// Carry.
    carry: Option<bool>,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for BicImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110x00001xxxxx0xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
                let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
                Self {
                    rd,
                    rn,
                    imm32,
                    carry,
                    set_flags: ins.bit(20),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let result = proc[self.rn] & !self.imm32;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "bic".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rn, self.imm32)
    }
}

/// BIC (register) instruction.
pub struct BicReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for BicReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0100001110xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11101010001xxxxx(0)xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rdn = ins.reg3(0);
                Self {
                    rd: rdn,
                    rn: rdn,
                    rm: ins.reg3(3),
                    shift: Shift::lsl(0),
                    set_flags: !state.in_it_block(),
                    encoding,
                }
            }
            T2 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
                let shift = Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6));
                Self {
                    rd,
                    rn,
                    rm,
                    shift,
                    set_flags: ins.bit(20),
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::ArmProcessor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let (shifted, carry) = shift_c(proc[self.rm], self.shift, carry_in);
        let result = proc[self.rn] & !shifted;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "bic".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, self.rn, self.encoding == T1),
            self.rm,
            self.shift.arg_string()
        )
    }
}
