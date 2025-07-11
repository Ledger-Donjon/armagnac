//! Implements LDRD (Load Register Dual) instruction.

use super::Encoding::{self, T1};
use super::{AddOrSub, Instruction};
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{indexing_args, other, unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

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
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "1110100xx1x1xxxxxxxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
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

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        let value = proc.read_u32_aligned(address)?;
        proc.set(self.rt, value);
        let value = proc.read_u32_aligned(address.wrapping_add(4))?;
        proc.set(self.rt2, value);
        if self.wback {
            proc.set(self.rn, offset_addr);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrd".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}",
            self.rt,
            self.rt2,
            indexing_args(self.rn, self.imm32, false, self.index, self.add, self.wback)
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
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "1110100xx1x11111xxxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
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

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        if proc.pc() % 4 != 0 {
            return Err(RunError::InstructionUnpredictable);
        }
        let address = proc.pc().wrapping_add_or_sub(self.imm32, self.add);
        let value = proc.read_u32_aligned(address)?;
        proc.set(self.rt, value);
        let value = proc.read_u32_aligned(address.wrapping_add(4))?;
        proc.set(self.rt2, value);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrd".into()
    }

    fn args(&self, _pc: u32) -> String {
        //let address = pc.wrapping_add(4).wrapping_add_or_sub(self.imm32, self.add);
        format!(
            "{}, {}, {}",
            self.rt,
            self.rt2,
            indexing_args(RegisterIndex::Pc, self.imm32, false, true, self.add, false)
        )
    }
}
