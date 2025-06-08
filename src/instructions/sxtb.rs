//! Implements SXTB (Signed Extend Byte) instruction.

use super::Encoding::{self, T1, T2};
use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arith::Shift;
use crate::qualifier_wide_match;
use crate::{
    arith::ror,
    core::ItState,
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    registers::RegisterIndex,
};

/// SXTB instruction.
///
/// Signed Extend Byte.
pub struct Sxtb {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rm: RegisterIndex,
    /// Rotation applied to Rm.
    /// Can be 0, 8, 16 or 24.
    rotation: u8,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Sxtb {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011001001xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11111010010011111111xxxx1(0)xxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                rotation: 0,
                encoding,
            },
            T2 => {
                let rm = ins.reg4(0);
                let rd = ins.reg4(8);
                unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rm,
                    rotation: (ins.imm2(4) << 3) as u8,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let rotated = ror(proc[self.rm], self.rotation as u32);
        proc.set(self.rd, ((rotated as i8) as i32) as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "sxtb".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            self.rd,
            self.rm,
            Shift::ror(self.rotation as u32).arg_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{ArmProcessor, Config},
        instructions::{sxtb::Sxtb, Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_sxtb() {
        let mut proc = ArmProcessor::new(Config::v8m());
        proc.registers.r1 = 0x12b456f8;

        let mut ins = Sxtb {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
            rotation: 0,
            encoding: DontCare,
        };

        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffffff8);

        ins.rotation = 8;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x00000056);

        ins.rotation = 16;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffffb4);

        ins.rotation = 24;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x00000012);
    }
}
