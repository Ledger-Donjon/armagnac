//! Implements LDRD (immediate) and LDRD (literal) instructions.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{indexing_args, other, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{AddOrSub, Instruction};

/// LDRD (immediate) instruction.
pub struct LdrdImm {
    /// First destination register.
    rt: RegisterIndex,
    /// Second destination register.
    rt2: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset from base.
    imm32: u32,
    /// True to load with indexing.
    pub index: bool,
    /// True to add offset, false to subtract.
    pub add: bool,
    /// True to write new offset value back to Rn.
    pub wback: bool,
}

impl Instruction for LdrdImm {
    fn patterns() -> &'static [&'static str] {
        &["1110100xx1x1xxxxxxxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let index = ins.bit(24);
        let add = ins.bit(23);
        let wback = ins.bit(21);
        let rt = ins.reg4(12);
        let rt2 = ins.reg4(8);
        let rn = ins.reg4(16);
        other(!index && !wback)?;
        other(rn.is_pc())?; // LDRD (literal)
        unpredictable(wback && (rn == rt || rn == rt2))?;
        unpredictable(rt.is_sp_or_pc() || rt2.is_sp_or_pc() || rt == rt2)?;
        Ok(Self {
            rt,
            rt2,
            rn,
            imm32: ins.imm8(0) << 2,
            index,
            add,
            wback,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        proc.registers[self.rt] = proc.u32le_at(address)?;
        proc.registers[self.rt2] = proc.u32le_at(address.wrapping_add(4))?;
        if self.wback {
            proc.registers[self.rn] = offset_addr;
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrd".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}",
            self.rt,
            self.rt2,
            indexing_args(self.rn, self.imm32, self.index, self.add, self.wback)
        )
    }
}

/// LDRD (literal) instruction.
pub struct LdrdLit {
    /// First destination register.
    rt: RegisterIndex,
    /// Second destination register.
    rt2: RegisterIndex,
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    pub add: bool,
}

impl Instruction for LdrdLit {
    fn patterns() -> &'static [&'static str] {
        &["1110100xx1x11111xxxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let index = ins.bit(24);
        let add = ins.bit(23);
        let wback = ins.bit(21);
        let rt = ins.reg4(12);
        let rt2 = ins.reg4(8);
        other(!index && !wback)?;
        unpredictable(rt.is_sp_or_pc() || rt2.is_sp_or_pc() || rt == rt2)?;
        unpredictable(wback)?;
        Ok(Self {
            rt,
            rt2,
            imm32: ins.imm8(0) << 2,
            add,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        if proc.pc() % 4 != 0 {
            return Err(RunError::InstructionUnpredictable);
        }
        let address = proc.pc().wrapping_add_or_sub(self.imm32, self.add);
        proc.registers[self.rt] = proc.u32le_at(address)?;
        proc.registers[self.rt2] = proc.u32le_at(address.wrapping_add(4))?;
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrd".into()
    }

    fn args(&self, pc: u32) -> String {
        let address = pc.wrapping_add(4).wrapping_add_or_sub(self.imm32, self.add);
        format!("{}, {}, [pc, #{}]", self.rt, self.rt2, address)
    }
}
