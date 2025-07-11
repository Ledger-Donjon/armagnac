//! Implements SXTH (Signed Extend Halfword) instruction.

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
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    registers::RegisterIndex,
};

/// SXTH instruction.
///
/// Unsigned Extend Halfword.
pub struct Sxth {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Rotation applied to Rm.
    /// Can be 0, 8, 16 or 24.
    rotation: u8,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Sxth {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011001000xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11111010000011111111xxxx1(0)xxxxxx",
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

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rotated = ror(proc[self.rm], self.rotation as u32);
        proc.set(self.rd, (((rotated & 0xffff) as i16) as i32) as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "sxth".into()
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
        core::{Config, Processor},
        instructions::{sxth::Sxth, Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_sxth() {
        let mut proc = Processor::new(Config::v8m());
        proc.registers.r1 = 0x12b456f8;

        let mut ins = Sxth {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
            rotation: 0,
            encoding: DontCare,
        };

        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x56f8);

        ins.rotation = 8;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffb456);

        ins.rotation = 16;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x12b4);

        ins.rotation = 24;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffff812);
    }
}
