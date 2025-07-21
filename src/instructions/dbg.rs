//! Implements DBG (Debug Hint) instruction.

use super::Encoding::{self, T1};
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Instruction, Pattern,
};
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    instructions::DecodeHelper,
};

/// DBG instruction.
///
/// Debug Hint.
///
/// When executed, Armagnac will returns [crate::arm::Event::DebugHint] event allowing the user to
/// catch such instruction.
pub struct Dbg {
    /// Extra information about the hint.
    /// In the range [0, 15].
    option: u8,
}

impl Instruction for Dbg {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111100111010(1)(1)(1)(1)10(0)0(0)0001111xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {
            option: ins.imm4(0) as u8,
        })
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::DebugHint(self.option))
    }

    fn name(&self) -> String {
        "dbg".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("#{}", self.option)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{Config, Effect, Processor},
        instructions::{dbg::Dbg, Instruction},
    };

    #[test]
    fn test_dbg() {
        let mut proc = Processor::new(Config::v7m());
        let result = Dbg { option: 0xa }.execute(&mut proc).unwrap();
        assert_eq!(result, Effect::DebugHint(0xa));
    }
}
