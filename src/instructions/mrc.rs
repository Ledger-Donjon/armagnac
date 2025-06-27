//! Implements MRC and MRC2 (Move to Arm Register from Coprocessor) instructions.

use super::{
    Encoding::{self, T1, T2},
    Instruction, Pattern,
};
use crate::{
    core::ItState,
    core::{
        Processor,
        ArmVersion::{V7EM, V7M, V8M},
        Effect, RunError,
    },
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// MRC or MRC2 instruction.
///
/// Move to Arm Register from Coprocessor.
pub struct Mrc {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor-specific opcode, from 0 to 7.
    opc1: u8,
    /// Destination Arm register.
    rt: RegisterIndex,
    /// First operand coprocessor register, from 0 to 15.
    crn: u8,
    /// Additional coprocessor source or destination register, from 0 to 15.
    crm: u8,
    /// Coprocessor-specific opcode, from 0 to 7.
    opc2: u8,
    /// Encoding.
    encoding: Encoding,
    /// Raw encoding.
    /// Required since transmitted directly to the coprocessor.
    ins: u32,
}

impl Instruction for Mrc {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "11101110xxx1xxxxxxxxxxxxxxx1xxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11111110xxx1xxxxxxxxxxxxxxx1xxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        let rt = ins.reg4(12);
        unpredictable(rt.is_sp())?;
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            opc1: ins.imm3(21) as u8,
            rt: ins.reg4(12),
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

        let value = coprocessor.borrow_mut().get_one_word(self.ins);
        if !self.rt.is_pc() {
            proc.set(self.rt, value);
        } else {
            proc.registers
                .psr
                .set_n(value.bit(31))
                .set_z(value.bit(30))
                .set_c(value.bit(29))
                .set_v(value.bit(28));
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        match self.encoding {
            Encoding::T1 => "mrc",
            Encoding::T2 => "mrc2",
            _ => panic!(),
        }
        .into()
    }

    fn args(&self, _pc: u32) -> String {
        let rt_str = if self.rt.is_pc() {
            "apsr_nzcv".into()
        } else {
            self.rt.to_string()
        };
        format!(
            "p{}, #{}, {}, c{}, c{}, #{}",
            self.coproc, self.opc1, rt_str, self.crn, self.crm, self.opc2
        )
    }
}
