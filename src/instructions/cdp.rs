//! Implements CDP and CDP2 (Coprocessor Data Processing) instructions.

use super::{
    Encoding::{self, T1, T2},
    Instruction, Pattern,
};
use crate::{
    core::ItState,
    core::{
        ArmProcessor,
        ArmVersion::{V7EM, V7M, V8M},
        Effect, RunError,
    },
    decoder::DecodeError,
    instructions::DecodeHelper,
};

/// CDP or CDP2 instruction.
///
/// Coprocessor Data Processing.
pub struct Cdp {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor-specific opcode, from 0 to 15.
    opc1: u8,
    /// Coprocessor destination register, from 0 to 15.
    crd: u8,
    /// First operand coprocessor register, from 0 to 15.
    crn: u8,
    /// Second operand coprocessor register, from 0 to 15.
    crm: u8,
    /// Coprocessor-specific opcode, from 0 to 7.
    opc2: u8,
    /// Encoding.
    encoding: Encoding,
    /// Raw encoding.
    /// Required since transmitted directly to the coprocessor.
    ins: u32,
}

impl Instruction for Cdp {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "11101110xxxxxxxxxxxxxxxxxxx0xxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11111110xxxxxxxxxxxxxxxxxxx0xxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            opc1: ins.imm4(20) as u8,
            crd: ins.imm4(12) as u8,
            crn: ins.imm4(16) as u8,
            crm: ins.imm4(0) as u8,
            opc2: ins.imm3(5) as u8,
            encoding,
            ins,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let Some(coprocessor) = proc.coproc_accepted(self.coproc, self.ins) else {
            proc.generate_coprocessor_exception();
            return Ok(Effect::None);
        };

        coprocessor.borrow_mut().internal_operation(self.ins);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        match self.encoding {
            T1 => "cdp",
            T2 => "cdp2",
            _ => panic!(),
        }
        .into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "p{}, #{}, c{}, c{}, c{}, #{}",
            self.coproc, self.opc1, self.crd, self.crn, self.crm, self.opc2
        )
    }
}
