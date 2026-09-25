//! Implements UXTAB (Unsigned Extend and Add Byte) instruction.
//!
//! UXTAB extracts an 8-bit value from a register, zero-extends it to 32 bits,
//! adds it to a value in another register, and writes the result to the destination register.
//! An optional rotation can be applied to the source register before extracting the byte.

use super::Encoding::{self, T1};
use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::arith::Shift;
use crate::qualifier_wide_match;
use crate::{
    arith::ror,
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    registers::RegisterIndex,
};

/// UXTAB instruction.
pub struct Uxtab {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register (value to add to).
    rn: RegisterIndex,
    /// Second operand register (byte to extract from).
    rm: RegisterIndex,
    /// Rotation applied to Rm before byte extraction.
    rotation: u8,
}

impl Instruction for Uxtab {
    fn patterns() -> &'static [Pattern] {
        &[
            // T1 encoding: UXTAB<c> <Rd>, <Rn>, <Rm>{, <rotation>}
            // 11111010010xxxxx1111xxxx1(0)xxxxxx
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "11111010010xxxxx1111xxxx1(0)xxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                unpredictable(rd.is_sp_or_pc() || rn.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    rotation: (ins.imm2(4) << 3) as u8,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rotated = ror(proc[self.rm], self.rotation as u32);
        let result = proc[self.rn].wrapping_add(rotated & 0xFF);
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "uxtab".into()
    }

    fn qualifier(&self) -> Qualifier {
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}{}",
            self.rd,
            self.rn,
            self.rm,
            Shift::ror(self.rotation as u32).arg_string()
        )
    }
}
