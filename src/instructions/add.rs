//! Implements ADD instruction.

use super::ArmVersion::{V6M, V7EM, V7M, V8M};
use super::Encoding::{self, T1, T2, T3, T4};
use super::{other, unpredictable, DecodeHelper, Instruction, Pattern, Qualifier};
use crate::qualifier_wide_match;
use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift, ShiftType},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::rdn_args_string,
    it_state::ItState,
    registers::RegisterIndex,
};
use core::panic;

/// ADD (immediate) instruction.
pub struct AddImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Value to be added.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for AddImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0001110xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "00110xxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x01000xxxxx0xxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T4,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x100000xxxx0xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<AddImm, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: ins.imm3(6),
                set_flags: !state.in_it_block(),
                encoding,
            },
            T2 => {
                let rdn = ins.reg3(8);
                Self {
                    rd: rdn,
                    rn: rdn,
                    imm32: ins & 0xff,
                    set_flags: !state.in_it_block(),
                    encoding,
                }
            }
            T3 => {
                let set_flags = ins.bit(20);
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff;
                let imm32 = thumb_expand_imm(imm12)?;
                other(rd.is_pc() && set_flags)?;
                other(rn.is_sp())?; // ADD (SP plus immediate)
                unpredictable(rd.is_sp_or_pc() || rn.is_pc())?;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags,
                    encoding,
                }
            }
            T4 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                if rn.is_sp_or_pc() {
                    return Err(DecodeError::Other); // ADR or ADD (SP plus immediate)
                }
                if rd.is_sp_or_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                Self {
                    rd,
                    rn,
                    imm32: (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins & 0xff,
                    set_flags: false,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let (r, c, v) = add_with_carry(proc[self.rn], self.imm32, false);
        proc.set(self.rd, r);
        if self.set_flags {
            proc.registers.psr.set_nz(r).set_c(c).set_v(v);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.encoding == T4 { "addw" } else { "add" }.into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> super::Qualifier {
        qualifier_wide_match!(self.encoding, T3)
    }

    fn args(&self, _pc: u32) -> String {
        let rdn = rdn_args_string(self.rd, self.rn, self.encoding == T2);
        format!("{rdn}, #{}", self.imm32)
    }
}

/// ADD (register) instruction.
pub struct AddReg {
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
    encoding: Encoding,
}

impl Instruction for AddReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0001100xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "01000100xxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11101011000xxxxx(0)xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
                set_flags: !state.in_it_block(),
                encoding,
            },
            T2 => {
                let rm = ins.reg4(3);
                let rdn = RegisterIndex::new_main((ins.imm1(7) << 3) | ins.imm3(0));
                if rdn.is_sp() || rm.is_sp() {
                    return Err(DecodeError::Other); // ADD (SP plus register)
                }
                if rdn.is_pc() && state.in_it_block_not_last() {
                    return Err(DecodeError::Unpredictable);
                }
                if rdn.is_pc() && rm.is_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                Self {
                    rd: rdn,
                    rn: rdn,
                    rm,
                    shift: Shift::lsl(0),
                    set_flags: false,
                    encoding,
                }
            }
            T3 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                let set_flags = ins.bit(20);
                if rd.is_pc() && set_flags {
                    return Err(DecodeError::Other); // CMN (register)
                }
                if rn.is_sp() {
                    return Err(DecodeError::Other); // ADD (SP plus register)
                }
                if rd.is_sp_or_pc() || rn.is_pc() || rm.is_sp_or_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                let shift = Shift::from_bits((ins >> 4) & 3, (ins.imm3(12) << 2) | ins.imm2(6));
                Self {
                    rd,
                    rn,
                    rm,
                    shift,
                    set_flags,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let (shifted, _) = shift_c(proc[self.rm], self.shift, carry_in);
        let (r, c, v) = add_with_carry(proc[self.rn], shifted, false);
        if self.rd.is_pc() {
            proc.alu_write_pc(r);
            Ok(true)
        } else {
            proc.set(self.rd, r);
            if self.set_flags {
                proc.registers.psr.set_nz(r).set_c(c).set_v(v);
            }
            Ok(false)
        }
    }

    fn name(&self) -> String {
        "add".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T3)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, self.rn, self.encoding == T2),
            self.rm,
            self.shift.arg_string()
        )
    }
}

/// ADD (SP plus immediate) instruction.
pub struct AddSpPlusImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Value to be added.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for AddSpPlusImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "10101xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "101100000xxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x01000x11010xxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T4,
                versions: &[V7M, V7EM, V8M],
                expression: "11110x10000011010xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(8),
                imm32: ins.imm8(0) << 2,
                set_flags: false,
                encoding,
            },
            T2 => Self {
                rd: RegisterIndex::Sp,
                imm32: ins.imm7(0) << 2,
                set_flags: false,
                encoding,
            },
            T3 => {
                let rd = ins.reg4(8);
                let set_flags = ins.bit(20);
                other(rd.is_pc() && set_flags)?; // CMN (immediate)
                unpredictable(rd.is_pc())?;
                let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0);
                let imm32 = thumb_expand_imm(imm12)?;
                Self {
                    rd,
                    imm32,
                    set_flags,
                    encoding,
                }
            }
            T4 => {
                let rd = ins.reg4(8);
                if rd.is_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                Self {
                    rd,
                    imm32: (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0),
                    set_flags: false,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::ArmProcessor) -> Result<bool, RunError> {
        let (result, carry, overflow) = add_with_carry(proc.sp(), self.imm32, false);
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
        if self.encoding == T4 { "addw" } else { "add" }.into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T3)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, #{}",
            rdn_args_string(self.rd, RegisterIndex::Sp, self.encoding == T2),
            self.imm32
        )
    }
}

/// ADD (SP plus register) instruction.
pub struct AddSpPlusReg {
    /// Destination register.
    rd: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift applied to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for AddSpPlusReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "01000100x1101xxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "010001001xxxx101",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11101011000x11010xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rdm = RegisterIndex::new_main((ins.imm1(7) << 3) | ins.imm3(0));
                Self {
                    rd: rdm,
                    rm: rdm,
                    shift: Shift::lsl(0),
                    set_flags: false,
                    encoding,
                }
            }
            T2 => {
                let rm = ins.reg4(3);
                other(rm.is_sp())?; // T1 encoding
                Self {
                    rd: RegisterIndex::Sp,
                    rm,
                    shift: Shift::lsl(0),
                    set_flags: false,
                    encoding,
                }
            }
            T3 => {
                let rd = ins.reg4(8);
                let rm = ins.reg4(0);
                let imm5 = (ins.imm3(12) << 2) | ins.imm2(6);
                let shift = Shift::from_bits(ins.imm2(4), imm5);
                unpredictable(rd.is_sp() && (shift.t != ShiftType::Lsl || shift.n > 3))?;
                unpredictable(rd.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rm,
                    shift,
                    set_flags: ins.bit(20),
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let (shifted, _) = shift_c(proc[self.rm], self.shift, carry_in);
        let (result, carry, overflow) = add_with_carry(proc.sp(), shifted, false);
        if self.rd.is_pc() {
            proc.alu_write_pc(result);
            Ok(true)
        } else {
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
    }

    fn name(&self) -> String {
        "add".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T3)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, RegisterIndex::Sp, self.encoding == T2),
            self.rm,
            self.shift.arg_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::AddImm;
    use crate::{
        arith::Shift,
        arm::{ArmProcessor, Config},
        instructions::{
            add::{AddReg, AddSpPlusImm, AddSpPlusReg},
            Encoding::DontCare,
            Instruction,
        },
        registers::RegisterIndex,
    };

    #[test]
    fn test_add_imm() {
        struct Test {
            rn_value: u32,
            imm32: u32,
            set_flags: bool,
            expected_rd_value: u32,
            expected_flags: u8,
        }

        let vectors = [
            Test {
                rn_value: 10,
                imm32: 20,
                set_flags: false,
                expected_rd_value: 30,
                expected_flags: 0,
            },
            Test {
                rn_value: 10,
                imm32: 20,
                set_flags: false,
                expected_rd_value: 30,
                expected_flags: 0,
            },
            Test {
                rn_value: 0xfffffffe,
                imm32: 3,
                set_flags: false,
                expected_rd_value: 1,
                expected_flags: 0,
            },
            Test {
                rn_value: 0xfffffffe,
                imm32: 1,
                set_flags: true,
                expected_rd_value: 0xffffffff,
                expected_flags: 0b10000,
            },
            Test {
                rn_value: 0xfffffffe,
                imm32: 2,
                set_flags: true,
                expected_rd_value: 0,
                expected_flags: 0b01100,
            },
            Test {
                rn_value: 0xfffffffe,
                imm32: 3,
                set_flags: true,
                expected_rd_value: 1,
                expected_flags: 0b00100,
            },
            Test {
                rn_value: 0x7fffffff,
                imm32: 1,
                set_flags: true,
                expected_rd_value: 0x80000000,
                expected_flags: 0b10010,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v8m());
            let rn = RegisterIndex::new_general_random();
            let rd = RegisterIndex::new_general_random();
            proc.set(rn, v.rn_value);
            let mut expected_registers = proc.registers.clone();
            AddImm {
                rd,
                rn,
                imm32: v.imm32,
                set_flags: v.set_flags,
                encoding: DontCare,
            }
            .execute(&mut proc)
            .unwrap();
            expected_registers.set(rd, v.expected_rd_value);
            expected_registers.psr.set_flags(v.expected_flags);
            assert_eq!(proc.registers, expected_registers);
        }
    }

    #[test]
    fn test_add_reg() {
        struct Test {
            rn_value: u32,
            rm_value: u32,
            shift: Shift,
            set_flags: bool,
            expected_rd_value: u32,
            expected_flags: u8,
        }

        let vectors = [
            Test {
                rn_value: 10,
                rm_value: 20,
                shift: Shift::lsl(0),
                set_flags: false,
                expected_rd_value: 30,
                expected_flags: 0,
            },
            Test {
                rn_value: 10,
                rm_value: 10,
                shift: Shift::lsl(2),
                set_flags: false,
                expected_rd_value: 50,
                expected_flags: 0,
            },
            Test {
                rn_value: 0xfffffffe,
                rm_value: 12,
                shift: Shift::lsr(2),
                set_flags: false,
                expected_rd_value: 1,
                expected_flags: 0,
            },
            Test {
                rn_value: 0xfffffffe,
                rm_value: 16,
                shift: Shift::lsr(4),
                set_flags: true,
                expected_rd_value: 0xffffffff,
                expected_flags: 0b10000,
            },
            Test {
                rn_value: 0xfffffffe,
                rm_value: 8,
                shift: Shift::ror(2),
                set_flags: true,
                expected_rd_value: 0,
                expected_flags: 0b01100,
            },
            Test {
                rn_value: 0xfffffffe,
                rm_value: 768,
                shift: Shift::lsr(8),
                set_flags: true,
                expected_rd_value: 1,
                expected_flags: 0b00100,
            },
            Test {
                rn_value: 0x7fffffff,
                rm_value: 1,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_rd_value: 0x80000000,
                expected_flags: 0b10010,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v8m());
            let rd = RegisterIndex::new_general_random();
            let (rn, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rd, 0);
            proc.set(rn, v.rn_value);
            proc.set(rm, v.rm_value);
            let mut expected_registers = proc.registers.clone();
            AddReg {
                rd,
                rn,
                rm,
                set_flags: v.set_flags,
                shift: v.shift,
                encoding: DontCare,
            }
            .execute(&mut proc)
            .unwrap();
            expected_registers.set(rd, v.expected_rd_value);
            expected_registers.psr.set_flags(v.expected_flags);
            assert_eq!(proc.registers, expected_registers);
        }
    }

    #[test]
    fn test_add_sp_plus_imm() {
        struct Test {
            imm32: u32,
            set_flags: bool,
            sp_value: u32,
            expected_rd_value: u32,
            expected_flags: u8,
        }

        let vectors = [
            Test {
                imm32: 20,
                set_flags: false,
                sp_value: 1000,
                expected_rd_value: 1020,
                expected_flags: 0,
            },
            Test {
                imm32: 1000,
                set_flags: false,
                sp_value: 0xfffffc18,
                expected_rd_value: 0,
                expected_flags: 0,
            },
            Test {
                imm32: 1000,
                set_flags: true,
                sp_value: 0xfffffc18,
                expected_rd_value: 0,
                expected_flags: 0b01100,
            },
            Test {
                imm32: 1000,
                set_flags: true,
                sp_value: 0xfffffc17,
                expected_rd_value: 0xffffffff,
                expected_flags: 0b10000,
            },
            Test {
                imm32: 2,
                set_flags: true,
                sp_value: 0x7fffffff,
                expected_rd_value: 0x80000001,
                expected_flags: 0b10010,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v8m());
            let rd = RegisterIndex::new_general_random();
            proc.registers.msp = v.sp_value;
            let mut expected_registers = proc.registers.clone();
            AddSpPlusImm {
                rd,
                imm32: v.imm32,
                set_flags: v.set_flags,
                encoding: DontCare,
            }
            .execute(&mut proc)
            .unwrap();
            expected_registers.set(rd, v.expected_rd_value);
            expected_registers.psr.set_flags(v.expected_flags);
            assert_eq!(proc.registers, expected_registers);
        }
    }

    #[test]
    fn test_add_sp_plus_reg() {
        struct Test {
            shift: Shift,
            set_flags: bool,
            sp_value: u32,
            rm_value: u32,
            expected_rd_value: u32,
            expected_flags: u8,
        }

        let vectors = [
            Test {
                shift: Shift::lsl(0),
                set_flags: false,
                sp_value: 1000,
                rm_value: 20,
                expected_rd_value: 1020,
                expected_flags: 0,
            },
            Test {
                shift: Shift::lsl(0),
                set_flags: false,
                sp_value: 0xfffffc18,
                rm_value: 1000,
                expected_rd_value: 0,
                expected_flags: 0,
            },
            Test {
                shift: Shift::lsl(2),
                set_flags: true,
                sp_value: 0xfffffc18,
                rm_value: 250,
                expected_rd_value: 0,
                expected_flags: 0b01100,
            },
            Test {
                shift: Shift::lsr(2),
                set_flags: true,
                sp_value: 0xfffffc17,
                rm_value: 4000,
                expected_rd_value: 0xffffffff,
                expected_flags: 0b10000,
            },
            Test {
                shift: Shift::lsl(1),
                set_flags: true,
                sp_value: 0x7fffffff,
                rm_value: 1,
                expected_rd_value: 0x80000001,
                expected_flags: 0b10010,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v8m());
            let rd = RegisterIndex::new_general_random();
            let rm = RegisterIndex::new_general_random();
            proc.registers.msp = v.sp_value;
            proc.set(rm, v.rm_value);
            let mut expected_registers = proc.registers.clone();
            AddSpPlusReg {
                rd,
                rm,
                shift: v.shift,
                set_flags: v.set_flags,
                encoding: DontCare,
            }
            .execute(&mut proc)
            .unwrap();
            expected_registers.set(rd, v.expected_rd_value);
            expected_registers.psr.set_flags(v.expected_flags);
            assert_eq!(proc.registers, expected_registers);
        }
    }
}
