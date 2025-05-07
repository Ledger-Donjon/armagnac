//! Implements USAT (Unsigned Saturate) instruction.

use super::{Instruction, Pattern};
use crate::{
    arith::{shift_c, unsigned_sat_q, Shift},
    arm::{
        ArmProcessor,
        ArmVersion::{V7EM, V7M, V8M},
        RunError,
    },
    instructions::{other, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// USAT instruction.
///
/// Unsigned Saturate.
pub struct Usat {
    /// Destination register.
    rd: RegisterIndex,
    /// Bit position for saturation, in range 0 to 31.
    saturate_to: u8,
    /// Register containing the value to be saturated.
    rn: RegisterIndex,
    /// Shift applied to Rn.
    shift: Shift,
}

impl Instruction for Usat {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110(0)1110x0xxxx0xxxxxxxxx(0)xxxxx",
        }]
    }

    fn try_decode(
        tn: usize,
        ins: u32,
        _state: ItState,
    ) -> Result<Self, crate::decoder::DecodeError> {
        debug_assert_eq!(tn, 1);
        let sh = ins.imm1(21);
        let imm5 = (ins.imm3(12) << 2) | ins.imm2(6);
        other((sh == 1) && (imm5 == 0))?;
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
        Ok(Self {
            rd,
            saturate_to: ins.imm5(0) as u8,
            rn,
            shift: Shift::from_bits(sh << 1, imm5),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let operand = shift_c(proc[self.rn], self.shift, false).0;
        let (result, sat) = unsigned_sat_q(operand as i32 as i64, self.saturate_to);
        proc.set(self.rd, result as u32);
        if sat {
            proc.registers.psr.set_q(true);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "usat".into()
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
    use super::Usat;
    use crate::{
        arith::Shift,
        arm::{ArmProcessor, Config},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    #[test]
    fn test_usat() {
        let vectors = [
            (0, 6, Shift::lsl(0), 0, false),
            (10, 6, Shift::lsl(0), 10, false),
            (63, 6, Shift::lsl(0), 63, false),
            (64, 6, Shift::lsl(0), 63, true),
            (32, 6, Shift::lsl(1), 63, true),
            (0xffffffff, 32, Shift::lsl(0), 0, true),
            (-10i32 as u32, 8, Shift::lsl(0), 0, true),
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v7m());
            let rd = RegisterIndex::new_general_random();
            let rn = RegisterIndex::new_general_random();
            proc.set(rn, v.0);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.3);
            expected.psr.set_q(v.4);
            Usat {
                rd,
                saturate_to: v.1,
                rn,
                shift: v.2,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}
