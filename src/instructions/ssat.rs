//! Implements SSAT (Signed Saturate) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    arith::{shift_c, signed_sat_q, Shift},
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{other, unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// SSAT instruction.
///
/// Signed Saturate.
pub struct Ssat {
    /// Destination register.
    rd: RegisterIndex,
    /// Register containing the value to be saturated.
    rn: RegisterIndex,
    /// Bit position for saturation.
    /// In [1, 32].
    saturate_to: u8,
    /// Shift applied to Rn.
    shift: Shift,
}

impl Instruction for Ssat {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110(0)1100x0xxxx0xxxxxxxxx(0)xxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let imm5 = (ins.imm3(12) << 2) | (ins.imm2(6));
        other(ins.bit(21) && (imm5 == 0))?; // SSAT16 if DSP extension, undefined otherwise.
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rn,
            saturate_to: (ins.imm5(0) + 1) as u8,
            shift: Shift::from_bits(ins.imm1(21) << 1, imm5),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        // Shift can only be LSL or ASR, so we don't have to take carry in.
        let operand = shift_c(proc[self.rn], self.shift, false).0;
        let (result, sat) = signed_sat_q(operand as i32 as i64, self.saturate_to);
        proc.set(self.rd, result as u32);
        if sat {
            proc.registers.psr.set_q(true);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ssat".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, #{}, {}{}",
            self.rd,
            self.saturate_to,
            self.rn,
            self.shift.arg_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Ssat;
    use crate::{
        arith::Shift,
        core::{Processor, Config},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    #[test]
    fn test_ssat() {
        let vectors = [
            (0, 1, Shift::lsl(0), 0, false),
            (1, 1, Shift::lsl(0), 0, true),
            (-128i32 as u32, 8, Shift::lsl(0), -128i32 as u32, false),
            (-129i32 as u32, 8, Shift::lsl(0), -128i32 as u32, true),
            (127, 8, Shift::lsl(0), 127, false),
            (128, 8, Shift::lsl(0), 127, true),
            (0x7fffffff, 32, Shift::lsl(0), 0x7fffffff, false),
            (0x7fffffff, 31, Shift::lsl(0), 0x3fffffff, true),
            (0x80000000, 31, Shift::lsl(0), 0xc0000000, true),
            (10, 8, Shift::lsl(1), 20, false),
            (10, 8, Shift::lsl(2), 40, false),
            (10, 8, Shift::lsl(4), 127, true),
            (200, 8, Shift::asr(1), 100, false),
        ];

        for v in vectors {
            let mut proc = Processor::new(Config::v7m());
            let (rd, rn) = RegisterIndex::pick_two_general_distinct();
            proc.set(rn, v.0);
            proc.registers.psr.set_q(v.4);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.3);
            Ssat {
                rd,
                rn,
                saturate_to: v.1,
                shift: v.2,
            }
            .execute(&mut proc)
            .unwrap();
            debug_assert_eq!(proc.registers, expected);
        }
    }
}
