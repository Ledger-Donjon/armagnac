//! Implements MCRR and MCRR2 (Move to Coprocessor from two Arm Registers) instructions.

use super::{
    Encoding::{self, T1, T2},
    Instruction, Pattern,
};
use crate::{
    core::ItState,
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// MCRR or MCRR2 instruction.
///
/// Move to Coprocessor from two Arm Registers.
pub struct Mcrr {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor-specific opcode, from 0 to 15.
    opc1: u8,
    /// First Arm source register.
    rt: RegisterIndex,
    /// Second Arm source register.
    rt2: RegisterIndex,
    /// Additional coprocessor destination register, from 0 to 15.
    crm: u8,
    /// Encoding.
    encoding: Encoding,
    /// Raw encoding.
    /// Required since transmitted directly to the coprocessor.
    ins: u32,
}

impl Instruction for Mcrr {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "111011000100xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111111000100xxxxxxxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        let rt = ins.reg4(12);
        let rt2 = ins.reg4(16);
        unpredictable(rt.is_sp_or_pc() || rt2.is_sp_or_pc())?;
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            opc1: ins.imm4(4) as u8,
            rt: ins.reg4(12),
            rt2: ins.reg4(16),
            crm: ins.imm4(0) as u8,
            encoding,
            ins,
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let Some(coprocessor) = proc.coproc_accepted(self.coproc, self.ins) else {
            proc.generate_coprocessor_exception();
            return Ok(Effect::None);
        };

        coprocessor
            .borrow_mut()
            .send_two_words(proc[self.rt2], proc[self.rt], self.ins);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        match self.encoding {
            Encoding::T1 => "mcrr",
            Encoding::T2 => "mcrr2",
            _ => panic!(),
        }
        .into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "p{}, #{}, {}, {}, c{}",
            self.coproc, self.opc1, self.rt, self.rt2, self.crm
        )
    }
}
