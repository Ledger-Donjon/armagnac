//! Implements CMP (Compare) instruction.

use super::Encoding::{self, T1, T2, T3};
use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    registers::RegisterIndex,
};

/// CMP (immediate) instruction.
pub struct CmpImm {
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand immediate value.
    imm32: u32,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for CmpImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "00101xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x011011xxxx0xxx1111xxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rn: ins.reg3(8),
                imm32: ins & 0xff,
                encoding,
            },
            T2 => Self {
                rn: ins.reg4(16),
                imm32: thumb_expand_imm((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff)?,
                encoding,
            },
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let (result, carry, overflow) = add_with_carry(proc[self.rn], !self.imm32, true);
        proc.registers
            .psr
            .set_nz(result)
            .set_c(carry)
            .set_v(overflow);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "cmp".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
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
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for CmpReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0100001010xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "01000101xxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "111010111011xxxx(0)xxx1111xxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rn: ins.reg3(0),
                rm: ins.reg3(3),
                shift: Shift::lsl(0),
                encoding,
            },
            T2 => {
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
                    encoding,
                }
            }
            T3 => {
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                unpredictable(rn.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let shifted = shift_c(proc[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc[self.rn], !shifted, true);
        proc.registers
            .psr
            .set_nz(result)
            .set_c(carry)
            .set_v(overflow);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "cmp".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T3)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}{}", self.rn, self.rm, self.shift.arg_string())
    }
}
