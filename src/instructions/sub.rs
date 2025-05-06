//! Implements SUB (Subtract) instruction.

use super::{other, unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arith::ShiftType;
use crate::qualifier_wide_match;
use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{rdn_args_string, ItState},
    registers::RegisterIndex,
};

/// SUB (immediate) instruction.
pub struct SubImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Value to be subtracted.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    tn: usize,
}

impl Instruction for SubImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0001111xxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "00111xxxxxxxxxxx",
            },
            Pattern {
                tn: 3,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x01101xxxxx0xxxxxxxxxxxxxxx",
            },
            Pattern {
                tn: 4,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x101010xxxx0xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: (ins >> 6) & 7,
                set_flags: !state.in_it_block(),
                tn,
            },
            2 => {
                let rdn = ins.reg3(8);
                Self {
                    rd: rdn,
                    rn: rdn,
                    imm32: ins & 0xff,
                    set_flags: !state.in_it_block(),
                    tn,
                }
            }
            3 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let set_flags = ins.bit(20);
                other(rd.is_pc() && set_flags)?; // CMP (immediate)
                other(rn.is_sp())?; // SUB (SP minus immediate)
                unpredictable(rd.is_sp_or_pc() || rn.is_pc())?;
                let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                let imm32 = thumb_expand_imm(imm12)?;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags,
                    tn,
                }
            }
            4 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // ADR
                other(rn.is_sp())?; // SUB (SP minus immediate)
                unpredictable(rd.is_sp_or_pc())?;
                let imm32 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags: false,
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let (result, carry, overflow) = add_with_carry(rn, !self.imm32, true);
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
        match self.tn {
            1 | 2 | 3 => "sub",
            4 => "subw",
            _ => panic!(),
        }
        .into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 3)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, #{}",
            rdn_args_string(self.rd, self.rn, self.tn == 2),
            self.imm32
        )
    }
}

/// SUB (register) instruction.
pub struct SubReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to apply to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    tn: usize,
}

impl Instruction for SubReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0001101xxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V7EM, V8M],
                expression: "11101011101xxxxx(0)xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
                set_flags: !state.in_it_block(),
                tn,
            },
            2 => {
                let rm = ins.reg4(0);
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let s = ins.bit(20);
                other(rd.is_pc() && s)?; // CMP (register)
                other(rn.is_sp())?; // SUB (SP minus register)
                unpredictable(rd.is_sp_or_pc() || rn.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
                    set_flags: s,
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let carry_in = proc.registers.psr.c();
        let (shifted, _) = shift_c(proc[self.rm], self.shift, carry_in);
        let (result, carry, overflow) = add_with_carry(rn, !shifted, true);
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
        "sub".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 2)
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

/// SUB (SP minus immediate) instruction.
pub struct SubSpMinusImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Value to be subtracted.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    tn: usize,
}

impl Instruction for SubSpMinusImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "101100001xxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x01101x1101xxxxxxxxxxxxxxxx",
            },
            Pattern {
                tn: 3,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x10101011010xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: RegisterIndex::Sp,
                imm32: (ins & 0x7f) << 2,
                set_flags: false,
                tn,
            },
            2 => {
                let rd = ins.reg4(8);
                let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                let imm32 = thumb_expand_imm(imm12)?;
                let set_flags = ins.bit(20);
                other(rd.is_pc() && set_flags)?; // CMP (immediate)
                unpredictable(rd.is_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: ins.bit(20),
                    tn,
                }
            }
            3 => {
                let rd = ins.reg4(8);
                let imm32 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                unpredictable(rd.is_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: ins.bit(20),
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let (result, carry, overflow) = add_with_carry(proc.sp(), !self.imm32, true);
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
        match self.tn {
            1 | 2 => "sub",
            3 => "subw",
            _ => panic!(),
        }
        .into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, #{}",
            rdn_args_string(self.rd, RegisterIndex::Sp, self.tn == 1),
            self.imm32
        )
    }
}

/// SUB (SP minus register) instruction.
///
/// Subtract.
pub struct SubSpMinusReg {
    /// Destination register.
    rd: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Shift applied to Rm.
    shift: Shift,
}

impl Instruction for SubSpMinusReg {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V7EM, V8M],
            expression: "11101011101x1101(0)xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rm = ins.reg4(0);
        let set_flags = ins.bit(20);
        other(rd.is_pc() && set_flags)?; // CMP (register)
        let shift = Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6));
        unpredictable(rd.is_sp() && (shift.t != ShiftType::Lsl || shift.n > 3))?;
        unpredictable((rd.is_pc() && !set_flags) || rm.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rm,
            set_flags,
            shift,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let shifted = shift_c(proc[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc.registers.sp(), !shifted, true);
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
        "sub".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}{}",
            self.rd,
            RegisterIndex::Sp,
            self.rm,
            self.shift.arg_string()
        )
    }
}
