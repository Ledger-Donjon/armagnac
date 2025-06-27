//! Implements LDREXB (Load Register Exclusive Byte) instruction.

use super::{
    Encoding::{self, T1},
    Pattern,
};
use crate::{
    core::{
        ArmProcessor,
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, RunError,
    },
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper, Instruction},
    registers::RegisterIndex,
};

/// LDREXB instruction.
///
/// Load Register Exclusive Byte.
pub struct Ldrexb {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
}

impl Instruction for Ldrexb {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111010001101xxxxxxxx(1)(1)(1)(1)0100(1)(1)(1)(1)",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rt = ins.reg4(12);
        let rn = ins.reg4(16);
        unpredictable(rt.is_sp_or_pc() || rn.is_pc())?;
        Ok(Self { rt, rn })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let address = proc[self.rn];
        proc.set_exclusive_monitors(address, 1);
        let value = proc.read_u8(address)? as u32;
        proc.set(self.rt, value);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrexb".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, [{}]", self.rt, self.rn)
    }
}
