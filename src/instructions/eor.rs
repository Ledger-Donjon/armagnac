//! Implements EOR (Exclusive OR) instruction.

use super::Encoding::{self, T1, T2};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use super::{Instruction, Qualifier};
use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{other, rdn_args_string, unpredictable, DecodeHelper},
    qualifier_wide_match,
    registers::RegisterIndex,
};

/// EOR (immediate) instruction
pub struct EorImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand immediate value.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Carry.
    carry: Option<bool>,
}

impl Instruction for EorImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110x00100xxxxx0xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let set_flags = ins.bit(20);
        let (imm32, carry) =
            thumb_expand_imm_optc((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0))?;
        other(rd.is_pc() && set_flags)?; // TEQ (immediate)
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rn,
            imm32,
            set_flags,
            carry,
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let result = proc[self.rn] ^ self.imm32;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "eor".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rn, self.imm32)
    }
}

/// EOR (register) instruction.
pub struct EorReg {
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

impl Instruction for EorReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0100000001xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11101010100xxxxx(0)xxxxxxxxxxxxxxx",
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
                Self {
                    rd,
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
                    set_flags: ins.bit(20),
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let (shifted, carry) = shift_c(proc[self.rm], self.shift, carry_in);
        let result = proc[self.rn] ^ shifted;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "eor".into()
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
