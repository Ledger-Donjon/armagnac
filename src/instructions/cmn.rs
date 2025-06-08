//! Implements CMN (Compare Negative) instruction.

use super::Encoding::{self, T1, T2};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use super::{Instruction, Qualifier};
use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    core::ItState,
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    qualifier_wide_match,
    registers::RegisterIndex,
};

/// CMN (immediate) instruction.
///
/// Compare Negative.
pub struct CmnImm {
    /// Operand register.
    rn: RegisterIndex,
    /// Immediate value to be added to rn.
    imm32: u32,
}

impl Instruction for CmnImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110x010001xxxx0xxx1111xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rn = ins.reg4(16);
        unpredictable(rn.is_pc())?;
        Ok(Self {
            rn,
            imm32: thumb_expand_imm((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0))?,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let (result, carry, overflow) = add_with_carry(proc.registers[self.rn], self.imm32, false);
        proc.registers
            .psr
            .set_nz(result)
            .set_c(carry)
            .set_v(overflow);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "cmn".into()
    }

    fn qualifier(&self) -> Qualifier {
        // Should be "cmn" but llvm-objdump prints "cmn.w" despite the Arm Architecture Reference
        // Manual does not say so.
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rn, self.imm32)
    }
}

/// CMN (register) instruction.
///
/// Compare Negative.
pub struct CmnReg {
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to apply to Rm.
    shift: Shift,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for CmnReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0100001011xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111010110001xxxx(0)xxx1111xxxxxxxx",
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
                let rm = ins.reg4(0);
                let rn = ins.reg4(16);
                let shift = Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6));
                unpredictable(rn.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rn,
                    rm,
                    shift,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let shifted = shift_c(proc[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc[self.rn], shifted, false);
        proc.registers
            .psr
            .set_nz(result)
            .set_c(carry)
            .set_v(overflow);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "cmn".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}{}", self.rn, self.rm, self.shift.arg_string())
    }
}

#[cfg(test)]
mod tests {
    use std::i32;

    use crate::{
        arith::Shift,
        core::{ArmProcessor, Config},
        instructions::{cmn::CmnImm, Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    use super::CmnReg;

    #[test]
    fn cmn_imm() {
        let mut proc = ArmProcessor::new(Config::v8m());
        cmm_imm_vec(&mut proc, -4, false, true, false);
        cmm_imm_vec(&mut proc, -5, true, true, false);
        cmm_imm_vec(&mut proc, -6, false, false, false);
        cmm_imm_vec(&mut proc, i32::MAX, false, false, true);
    }

    fn cmm_imm_vec(proc: &mut ArmProcessor, inital_rn: i32, z: bool, c: bool, v: bool) {
        let rn = RegisterIndex::new_general_random();
        let ins = CmnImm { rn, imm32: 5 };
        proc.registers.psr.set(0);
        proc.set(rn, inital_rn as u32);
        ins.execute(proc).unwrap();
        assert_eq!(proc.registers.psr.z(), z);
        assert_eq!(proc.registers.psr.c(), c);
        assert_eq!(proc.registers.psr.v(), v);
    }

    #[test]
    fn cmn_reg() {
        let mut proc = ArmProcessor::new(Config::v8m());
        cmn_reg_vec(&mut proc, -4, 5, Shift::lsl(0), false, true, false);
        cmn_reg_vec(&mut proc, -5, 5, Shift::lsl(0), true, true, false);
        cmn_reg_vec(&mut proc, -6, 5, Shift::lsl(0), false, false, false);
        cmn_reg_vec(&mut proc, i32::MAX, 5, Shift::lsl(0), false, false, true);
        cmn_reg_vec(&mut proc, -19, 5, Shift::lsl(2), false, true, false);
        cmn_reg_vec(&mut proc, -20, 5, Shift::lsl(2), true, true, false);
        cmn_reg_vec(&mut proc, -21, 5, Shift::lsl(2), false, false, false);
        cmn_reg_vec(&mut proc, -4, 20, Shift::lsr(2), false, true, false);
        cmn_reg_vec(&mut proc, -5, 20, Shift::lsr(2), true, true, false);
        cmn_reg_vec(&mut proc, -6, 20, Shift::lsr(2), false, false, false);
    }

    fn cmn_reg_vec(
        proc: &mut ArmProcessor,
        r0: i32,
        r1: u32,
        shift: Shift,
        z: bool,
        c: bool,
        v: bool,
    ) {
        let (rn, rm) = RegisterIndex::pick_two_general_distinct();
        proc.registers.psr.set(0);
        proc.set(rn, r0 as u32);
        proc.set(rm, r1);
        CmnReg {
            rn,
            rm,
            shift,
            encoding: DontCare,
        }
        .execute(proc)
        .unwrap();
        assert_eq!(proc.registers.psr.z(), z);
        assert_eq!(proc.registers.psr.c(), c);
        assert_eq!(proc.registers.psr.v(), v);
    }
}
