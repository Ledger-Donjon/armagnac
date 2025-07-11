//! Implements LDREXH (Load Register Exclusive Halfword) instruction.

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

/// LDREXH instruction.
///
/// Load Register Exclusive Halfword.
pub struct Ldrexh {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
}

impl Instruction for Ldrexh {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111010001101xxxxxxxx(1)(1)(1)(1)0101(1)(1)(1)(1)",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rt = ins.reg4(12);
        let rn = ins.reg4(16);
        unpredictable(rt.is_sp_or_pc() || rn.is_pc())?;
        Ok(Self { rt, rn })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let address = proc[self.rn];
        proc.set_exclusive_monitors(address, 2);
        let value = proc.read_u16_aligned(address)? as u32;
        proc.set(self.rt, value);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrexh".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, [{}]", self.rt, self.rn)
    }
}
