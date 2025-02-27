use crate::{
    arith::sign_extend,
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// SBFX instruction.
///
/// Signed Bit Field Extract.
pub struct Sbfx {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Least significant bit number in the bitfield.
    lsb: u8,
    /// Width of the bitfield minus 1.
    widthm1: u8,
}

impl Instruction for Sbfx {
    fn patterns() -> &'static [&'static str] {
        &["11110(0)110100xxxx0xxxxxxxxx(0)xxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() | rn.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rn,
            lsb: (ins.imm3(12) << 2 | ins.imm2(6)) as u8,
            widthm1: ins.imm5(0) as u8,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let msbit = self.lsb + self.widthm1;
        if msbit <= 31 {
            let result = sign_extend(proc.registers[self.rn] >> self.lsb, self.widthm1 + 1);
            proc.registers.set(self.rd, result as u32);
            Ok(false)
        } else {
            return Err(RunError::InstructionUnpredictable);
        }
    }

    fn name(&self) -> String {
        "sbfx".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, #{}, #{}",
            self.rd,
            self.rn,
            self.lsb,
            self.widthm1 + 1
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Sbfx;
    use crate::{arm::Arm7Processor, instructions::Instruction, registers::RegisterIndex};

    fn test_sbfx_vec(proc: &mut Arm7Processor, r1: u32, lsb: u8, widthm1: u8, expected_r0: u32) {
        proc.registers.r0 = 0;
        proc.registers.r1 = r1;
        Sbfx {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            lsb,
            widthm1,
        }
        .execute(proc)
        .unwrap();
        assert_eq!(proc.registers.r0, expected_r0);
    }

    #[test]
    fn test_sbfx() {
        let mut proc = Arm7Processor::new(crate::arm::ArmVersion::V8M, 0);
        let magic = 0x12b456f8;
        test_sbfx_vec(&mut proc, magic, 0, 0, 0);
        test_sbfx_vec(&mut proc, magic, 3, 0, 0xffffffff);
        test_sbfx_vec(&mut proc, magic, 0, 3, 0xfffffff8);
        test_sbfx_vec(&mut proc, magic, 24, 7, 0x12);
        test_sbfx_vec(&mut proc, magic, 16, 7, 0xffffffb4);
        test_sbfx_vec(&mut proc, magic, 8, 7, 0x56);
        test_sbfx_vec(&mut proc, magic, 0, 7, 0xfffffff8);
        test_sbfx_vec(&mut proc, magic, 8, 15, 0xffffb456);
        test_sbfx_vec(&mut proc, magic, 28, 3, 1);
        test_sbfx_vec(&mut proc, magic, 28, 0, 0xffffffff);
    }
}
