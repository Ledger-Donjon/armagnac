use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{unpredictable, DecodeHelper, Instruction};

/// REV16 instruction.
pub struct Rev16 {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rm: RegisterIndex,
}

impl Instruction for Rev16 {
    fn patterns() -> &'static [&'static str] {
        &["1011101001xxxxxx", "111110101001xxxx1111xxxx1001xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
            },
            2 => {
                let rm1 = ins.reg4(0);
                let rm2 = ins.reg4(16);
                let rd = ins.reg4(8);
                unpredictable(rm1 != rm2)?;
                unpredictable(rd.is_sp_or_pc() || rm1.is_sp_or_pc())?;
                Self { rd, rm: rm1 }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rm = proc.registers[self.rm];
        let result = (rm & 0x00ff00ff) << 8 | (rm & 0xff00ff00) >> 8;
        proc.registers.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "rev16".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::Arm7Processor,
        instructions::{rev16::Rev16, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_rev16() {
        let mut proc = Arm7Processor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 0x12345678;
        let ins = Rev16 {
            rd: RegisterIndex::R0,
            rm: RegisterIndex::R1,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x34127856);
    }
}
