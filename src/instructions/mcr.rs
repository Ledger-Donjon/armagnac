//! Implements MCR and MCR2 (Move to Coprocessor from Arm Register) instructions.

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

/// MCR or MCR2 instruction.
///
/// Move to Coprocessor from Arm Register.
pub struct Mcr {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor-specific opcode, from 0 to 7.
    opc1: u8,
    /// Arm Source register.
    rt: RegisterIndex,
    /// Coprocessor destination register, from 0 to 15.
    crn: u8,
    /// Additional coprocessor destination register, from 0 to 15.
    crm: u8,
    /// Coprocessor-specific opcode, from 0 to 7.
    opc2: u8,
    /// Encoding.
    encoding: Encoding,
    /// Raw encoding.
    /// Required since transmitted directly to the coprocessor.
    ins: u32,
}

impl Instruction for Mcr {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "11101110xxx0xxxxxxxxxxxxxxx1xxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11111110xxx0xxxxxxxxxxxxxxx1xxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        let rt = ins.reg4(12);
        unpredictable(rt.is_sp_or_pc())?;
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            opc1: ins.imm3(21) as u8,
            rt,
            crn: ins.imm4(16) as u8,
            crm: ins.imm4(0) as u8,
            opc2: ins.imm3(5) as u8,
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
            .send_one_word(proc[self.rt], self.ins);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        match self.encoding {
            Encoding::T1 => "mcr",
            Encoding::T2 => "mcr2",
            _ => panic!(),
        }
        .into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "p{}, #{}, {}, c{}, c{}, #{}",
            self.coproc, self.opc1, self.rt, self.crn, self.crm, self.opc2
        )
    }
}
