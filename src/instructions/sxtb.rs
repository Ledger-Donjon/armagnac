//! Implements SXTB (Signed Extend Byte) instruction.

use super::{unpredictable, DecodeHelper, Instruction};
use crate::{
    arith::ror,
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    it_state::ItState,
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
}

impl Instruction for Sxtb {
    fn patterns() -> &'static [&'static str] {
        &["1011001001xxxxxx", "11111010000011111111xxxx1(0)xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                rotation: 0,
            },
            2 => {
                let rm = ins.reg4(0);
                let rd = ins.reg4(8);
                unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rm,
                    rotation: (ins.imm2(4) << 3) as u8,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rotated = ror(proc.registers[self.rm], self.rotation as u32);
        proc.registers.set(self.rd, ((rotated as i8) as i32) as u32);
        Ok(false)
    }

    fn name(&self) -> String {
        "sxtb".into()
    }

    fn args(&self, _pc: u32) -> String {
        let ror_str = if self.rotation != 0 {
            format!(", ROR #{}", self.rotation)
        } else {
            "".into()
        };
        format!("{}, {}{}", self.rd, self.rm, ror_str)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::Arm7Processor,
        instructions::{sxtb::Sxtb, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_sxtb() {
        let mut proc = Arm7Processor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 0x12b456f8;

        let mut ins = Sxtb {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
            rotation: 0,
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
