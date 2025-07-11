//! Implements MRRC and MRRC2 (Move to two Arm Registers from Coprocessor) instructions.

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

/// MRRC or MRRC2 instruction.
///
/// Move to two Arm Registers from Coprocessor.
pub struct Mrrc {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor-specific opcode, from 0 to 15.
    opc: u8,
    /// First destination Arm register.
    rt: RegisterIndex,
    /// Second destination Arm register.
    rt2: RegisterIndex,
    /// Coprocessor register supplying the data, from 0 to 15.
    crm: u8,
    /// Encoding.
    encoding: Encoding,
    /// Raw encoding.
    /// Required since transmitted directly to the coprocessor.
    ins: u32,
}

impl Instruction for Mrrc {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "111011000101xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111111000101xxxxxxxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        let rt = ins.reg4(12);
        let rt2 = ins.reg4(16);
        unpredictable(rt.is_sp_or_pc() || rt2.is_sp_or_pc() || rt == rt2)?;
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            opc: ins.imm4(4) as u8,
            rt,
            rt2,
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

        let (rt2, rt) = coprocessor.borrow_mut().get_two_words(self.ins);
        proc.set(self.rt2, rt2);
        proc.set(self.rt, rt);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        match self.encoding {
            Encoding::T1 => "mrrc",
            Encoding::T2 => "mrrc2",
            _ => panic!(),
        }
        .into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "p{}, #{}, {}, {}, c{}",
            self.coproc, self.opc, self.rt, self.rt2, self.crm
        )
    }
}
