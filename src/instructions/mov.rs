//! Implements MOV (Move) instruction.

use super::Encoding::{self, T1, T2, T3};
use super::{other, unpredictable, DecodeHelper, Instruction};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arith::{shift_c, Shift, ShiftType};
use crate::qualifier_wide_match;
use crate::{
    arith::thumb_expand_imm_optc,
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::ItState,
    registers::RegisterIndex,
};
use core::panic;

/// MOV (immediate) instruction.
pub struct MovImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Immediate value to be placed in Rd.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Carry.
    carry: Option<bool>,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for MovImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "00100xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x00010x11110xxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x100100xxxx0xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(8),
                imm32: ins.imm8(0),
                set_flags: !state.in_it_block(),
                carry: None,
                encoding,
            },
            T2 => {
                let rd = ins.reg4(8);
                let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
                unpredictable(rd.is_sp_or_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: ins.bit(20),
                    carry,
                    encoding,
                }
            }
            T3 => {
                let rd = ins.reg4(8);
                let imm32 =
                    (ins.imm4(16) << 12) | (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                unpredictable(rd.is_sp_or_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: false,
                    carry: None,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        proc.set(self.rd, self.imm32);
        if self.set_flags {
            proc.registers.psr.set_nz(self.imm32).set_c_opt(self.carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        if self.encoding == T3 { "movw" } else { "mov" }.into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> super::Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rd, self.imm32)
    }
}

/// MOV (register) instruction.
pub struct MovReg {
    /// Destination register.
    rd: RegisterIndex,
    /// Source register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for MovReg {
    fn patterns() -> &'static [Pattern] {
        // TODO: For ArmV8-M, encodings T2 and T3 can support shifts, this is not implemented yet.
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "01000110xxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0000000000xxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11101010010x1111(0)000xxxx0000xxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rd = RegisterIndex::new_main((ins & 7) | (ins.imm1(7) << 3));
                unpredictable(rd.is_pc() && state.in_it_block_not_last())?;
                Self {
                    rd,
                    rm: ins.reg4(3),
                    set_flags: false,
                    encoding,
                }
            }
            T2 => {
                unpredictable(state.in_it_block())?;
                Self {
                    rd: ins.reg3(0),
                    rm: ins.reg3(3),
                    set_flags: true,
                    encoding,
                }
            }
            T3 => {
                let rd = ins.reg4(8);
                let rm = ins.reg4(0);
                let set_flags = ins.bit(20);
                unpredictable(set_flags && (rd.is_sp_or_pc() || rm.is_sp_or_pc()))?;
                unpredictable(
                    !set_flags && (rd.is_pc() || rm.is_pc() || (rd.is_sp() && rm.is_sp())),
                )?;
                Self {
                    rd,
                    rm,
                    set_flags,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let result = proc[self.rm];
        if self.rd.is_pc() {
            proc.alu_write_pc(result);
            Ok(Effect::Branch)
        } else {
            proc.set(self.rd, result);
            if self.set_flags {
                proc.registers.psr.set_nz(result);
            }
            Ok(Effect::None)
        }
    }

    fn name(&self) -> String {
        "mov".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> super::Qualifier {
        qualifier_wide_match!(self.encoding, T3)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

/// MOV (Register-shifted Register).
///
/// This is a ArmV8 instruction, similar to RRX instruction in ArmV7.
pub struct MovRegShiftReg {
    /// Destination register.
    rd: RegisterIndex,
    /// Source register.
    rm: RegisterIndex,
    /// Register holding the amount of shift.
    rs: RegisterIndex,
    /// Shift type.
    shift_type: ShiftType,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for MovRegShiftReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V8M],
                expression: "010000xxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V8M],
                expression: "111110100xxxxxxx1111xxxx0000xxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        match encoding {
            T1 => {
                let rdm = ins.reg3(0);
                let op = ins.imm4(6);
                other((op != 0b0010) && (op != 0b0011) && (op != 0b0100) && (op != 0b0111))?; // Related encodings
                Ok(Self {
                    rd: rdm,
                    rm: rdm,
                    rs: ins.reg3(3),
                    shift_type: ShiftType::from_bits(((op >> 1) & 2) | (op & 1)),
                    set_flags: !state.in_it_block(),
                })
            }
            T2 => {
                let rd = ins.reg4(8);
                let rm = ins.reg4(16);
                let rs = ins.reg4(0);
                unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc() || rs.is_sp_or_pc())?;
                Ok(Self {
                    rd,
                    rm,
                    rs,
                    shift_type: ShiftType::from_bits(ins.imm2(21)),
                    set_flags: ins.bit(20),
                })
            }
            _ => panic!(),
        }
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let shift_n = proc[self.rs] & 0xff;
        let carry_in = proc.registers.psr.c();
        let (result, carry) = shift_c(
            proc[self.rm],
            Shift {
                t: self.shift_type,
                n: shift_n,
            },
            carry_in,
        );
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "mov".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {} {}", self.rd, self.rm, self.shift_type, self.rs)
    }
}
