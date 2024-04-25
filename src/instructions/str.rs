//! Implements STR (immediate, register) and STRD (immediate) instructions.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::TestBit,
    instructions::{DecodeHelper, ItState},
    registers::RegisterIndex,
};

use super::{indexing_args, other, reg, undefined, unpredictable, AddOrSub, Instruction};

/// STR (immediate) instruction.
pub struct StrImm {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset.
    imm32: u32,
    /// True to store with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// True to write new offset value back to Rn.
    wback: bool,
}

impl Instruction for StrImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "01100xxxxxxxxxxx",
            "10010xxxxxxxxxxx",
            "111110001100xxxxxxxxxxxxxxxxxxxx",
            "111110000100xxxxxxxx1xxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: reg(ins & 7),
                rn: reg(ins >> 3 & 7),
                imm32: (ins >> 6 & 0x1f) << 2,
                index: true,
                add: true,
                wback: false,
            },
            2 => Self {
                rt: reg(ins >> 8 & 7),
                rn: RegisterIndex::Sp,
                imm32: (ins & 0xff) << 2,
                index: true,
                add: true,
                wback: false,
            },
            3 => {
                let rn = reg(ins >> 16 & 0xf);
                let rt = reg(ins >> 12 & 0xf);
                undefined(rn.is_pc())?;
                unpredictable(rt.is_pc())?;
                Self {
                    rt,
                    rn,
                    imm32: ins & 0xfff,
                    index: true,
                    add: true,
                    wback: false,
                }
            }
            4 => {
                let rn = reg(ins >> 16 & 0xf);
                let rt = reg(ins >> 12 & 0xf);
                let puw = ins >> 8 & 7;
                let imm32 = ins & 0xff;
                let wback = puw & 1 != 0;
                other(puw == 6)?; // STRT
                other(rn.is_sp() && puw == 5 && imm32 == 4)?; // PUSH
                undefined(rn.is_pc() || puw & 5 == 0)?;
                unpredictable(rt.is_pc() || (wback && rn == rt))?;
                Self {
                    rt,
                    rn,
                    imm32,
                    index: puw & 4 != 0,
                    add: puw & 2 != 0,
                    wback,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn].val();
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        proc.set_u32le_at(address, proc.registers[self.rt].val())?;
        if self.wback {
            proc.registers[self.rn].set_val(offset_addr)
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "str".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, self.index, self.add, self.wback,)
        )
    }
}

/// STRD (immediate) instruction.
pub struct StrdImm {
    /// First source register.
    rt: RegisterIndex,
    /// Second source register.
    rt2: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Immediate offset.
    imm32: u32,
    /// True to store with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// True to write new offset value back to Rn.
    wback: bool,
}

impl Instruction for StrdImm {
    fn patterns() -> &'static [&'static str] {
        &["1110100xx1x0xxxxxxxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rt = ins.reg4(12);
        let rt2 = ins.reg4(8);
        let rn = ins.reg4(16);
        let index = ins.bit(24);
        let add = ins.bit(23);
        let wback = ins.bit(21);
        unpredictable(wback && (rn == rt || rn == rt2))?;
        unpredictable(rn.is_pc() || rt.is_sp_or_pc() || rt2.is_sp_or_pc())?;
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
        let rn = proc[self.rn].val();
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        let rt = proc[self.rt].val();
        let rt2 = proc[self.rt2].val();
        proc.set_u32le_at(address, rt)?;
        proc.set_u32le_at(address.wrapping_add(4), rt2)?;
        if self.wback {
            proc[self.rn].set_val(offset_addr);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "strd".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}",
            self.rt,
            self.rt2,
            indexing_args(self.rn, self.imm32, self.index, self.add, self.wback,)
        )
    }
}
