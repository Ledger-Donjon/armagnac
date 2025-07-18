//! Implements STREXH (Store Register Exclusive Halfword) instruction.

use super::{
    Encoding::{self, T1},
    Pattern,
};
use crate::{
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper, Instruction},
    registers::RegisterIndex,
};

/// STREXH instruction.
///
/// Store Register Exclusive Halfword.
pub struct Strexh {
    /// Destination register.
    rd: RegisterIndex,
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
}

impl Instruction for Strexh {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111010001100xxxxxxxx(1)(1)(1)(1)0101xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(0);
        let rt = ins.reg4(12);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rt.is_sp_or_pc() || rn.is_pc())?;
        unpredictable(rd == rn || rd == rt)?;
        Ok(Self { rd, rt, rn })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let address = proc[self.rn];
        if proc.exclusive_monitors_pass(address, 2)? {
            let value = proc[self.rt] as u16;
            proc.write_u16_aligned(address, value)?;
            proc.set(self.rd, 0);
        } else {
            proc.set(self.rd, 1);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "strexh".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, [{}]", self.rd, self.rt, self.rn)
    }
}
