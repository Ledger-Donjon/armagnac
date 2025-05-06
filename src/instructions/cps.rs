//! Implements CPS (Change Processor State) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::unpredictable,
    it_state::ItState,
};

pub struct Cps {
    /// True when interrupt must be enabled, false otherwise.
    enable: bool,
    /// PRIMASK flag.
    affect_pri: bool,
    /// FAULTMAST flag.
    affect_fault: bool,
}

impl Instruction for Cps {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M],
                expression: "10110110011x(0)(0)(1)(0)",
            },
            Pattern {
                tn: 1,
                versions: &[V7M, V7EM, V8M],
                expression: "10110110011x(0)(0)xx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        unpredictable(state.in_it_block())?;
        Ok(Self {
            enable: (ins >> 4) & 1 == 0,
            affect_pri: (ins & 2) != 0,
            affect_fault: (ins & 1) != 0,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        if proc.is_privileged() {
            if self.enable {
                if self.affect_pri {
                    proc.registers.primask.set_pm(false);
                }
                if self.affect_fault {
                    proc.registers.faultmask.set_pm(false);
                }
            } else {
                if self.affect_pri {
                    proc.registers.primask.set_pm(true);
                }
                if self.affect_fault && proc.execution_priority >= 0 {
                    proc.registers.faultmask.set_pm(true);
                }
            }
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.enable { "cpsie" } else { "cpsid" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}{}",
            if self.affect_pri { "i" } else { "" },
            if self.affect_fault { "f" } else { "" }
        )
    }
}
