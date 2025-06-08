//! Implements ROR (Rotate Right) instruction.

use super::Encoding::{self, T1, T2};
use super::{
    ArmVersion::{V6M, V7EM, V7M},
    Pattern,
};
use super::{Instruction, Qualifier};
use crate::qualifier_wide_match;
use crate::{
    arith::{shift_c, Shift},
    core::ItState,
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{other, rdn_args_string, unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// ROR (immediate) instruction.
///
/// Rotate Right.
pub struct RorImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for RorImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM],
            expression: "11101010010x1111(0)xxxxxxxxx11xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let imm5 = (ins.imm3(12) << 2) | ins.imm2(6);
        other(imm5 == 0)?; // RRX
        let rd = ins.reg4(8);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
        let shift = Shift::from_bits(3, imm5);
        Ok(Self {
            rd,
            rm,
            shift,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let (result, carry) = shift_c(proc[self.rm], self.shift, carry_in);
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ror".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rm, self.shift.n)
    }
}

/// ROR (register) instruction.
///
/// Rotate Right.
pub struct RorReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Shift amount register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for RorReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM],
                expression: "0100000111xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM],
                expression: "11111010011xxxxx1111xxxx0000xxxx",
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
                    set_flags: ins.bit(20),
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let shift_n = proc[self.rm] & 0xff;
        let carry_in = proc.registers.psr.c();
        let (result, carry) = shift_c(proc[self.rn], Shift::ror(shift_n), carry_in);
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ror".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            rdn_args_string(self.rd, self.rn, self.encoding == T1),
            self.rm
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arith::Shift,
        core::{ArmProcessor, Config},
        instructions::{
            ror::{RorImm, RorReg},
            Encoding::DontCare,
            Instruction,
        },
        registers::RegisterIndex,
    };

    #[test]
    fn test_ror_imm() {
        let mut proc = ArmProcessor::new(Config::v8m());
        proc.registers.r1 = 0x12345678;
        let mut ins = RorImm {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
            shift: Shift::ror(1),
            set_flags: true,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x091a2b3c);
        assert_eq!(proc.registers.psr.c(), false);
        assert_eq!(proc.registers.psr.n(), false);
        ins.shift.n = 4;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x81234567);
        assert_eq!(proc.registers.psr.c(), true);
        assert_eq!(proc.registers.psr.n(), true);
    }

    #[test]
    fn test_ror_reg() {
        let mut proc = ArmProcessor::new(Config::v8m());
        proc.registers.r1 = 0x12345678;
        proc.registers.r2 = 1;
        let ins = RorReg {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            set_flags: true,
            encoding: DontCare,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x091a2b3c);
        assert_eq!(proc.registers.psr.c(), false);
        assert_eq!(proc.registers.psr.n(), false);
        proc.registers.r2 = 4;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x81234567);
        assert_eq!(proc.registers.psr.c(), true);
        assert_eq!(proc.registers.psr.n(), true);
    }
}
