//! Implements MVN (Move Not) instruction.

use super::Encoding::{self, T1, T2};
use super::{unpredictable, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::DecodeHelper,
    registers::RegisterIndex,
};

/// MVN (immediate) instruction.
pub struct MvnImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Immediate value
    imm32: u32,
    /// Carry
    carry: Option<bool>,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for MvnImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110x00011x11110xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let imm12 = (ins.imm1(26) << 11) | ((ins.imm3(12)) << 8) | ins.imm8(0);
        let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
        Ok(Self {
            rd: ins.reg4(8),
            imm32,
            carry,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let result = !self.imm32;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        if self.set_flags { "mvns" } else { "mvn" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rd, self.imm32)
    }
}

/// MVN (register) instruction.
pub struct MvnReg {
    /// Destination register.
    rd: RegisterIndex,
    /// Source register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for MvnReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0100001111xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11101010011x1111(0)xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                shift: Shift::lsl(0),
                set_flags: !state.in_it_block(),
                encoding,
            },
            T2 => {
                let rd = ins.reg4(8);
                let rm = ins.reg4(0);
                unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
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
        let result = !shifted;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "mvn".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}{}", self.rd, self.rm, self.shift.arg_string())
    }
}

#[cfg(test)]
mod tests {
    use super::MvnReg;
    use crate::{
        arith::Shift,
        core::{Processor, Config},
        instructions::{Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_mvn_reg() {
        let mut proc = Processor::new(Config::v8m());
        proc.registers.r0 = 0x10;
        proc.registers.r1 = 0x11;
        let ins = MvnReg {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
            shift: Shift::lsl(0),
            set_flags: false,
            encoding: DontCare,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffffee);
    }
}
