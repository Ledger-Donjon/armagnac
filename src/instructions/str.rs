//! Implements STR (Store Register) instruction.

use crate::{
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{DecodeHelper, ItState},
    registers::RegisterIndex,
};

use super::{indexing_args, other, undefined, unpredictable, AddOrSub, Instruction};

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
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: (ins >> 6 & 0x1f) << 2,
                index: true,
                add: true,
                wback: false,
            },
            2 => Self {
                rt: ins.reg3(8),
                rn: RegisterIndex::Sp,
                imm32: (ins & 0xff) << 2,
                index: true,
                add: true,
                wback: false,
            },
            3 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
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
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
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

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        proc.set_u32le_at(address, proc.registers[self.rt])?;
        if self.wback {
            proc.registers.set(self.rn, offset_addr)
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

/// STR (register) instruction.
pub struct StrReg {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift to apply to Rm.
    shift: Shift,
}

impl Instruction for StrReg {
    fn patterns() -> &'static [&'static str] {
        &["0101000xxxxxxxxx", "111110000100xxxxxxxx000000xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
            },
            2 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                let rm = ins.reg4(0);
                undefined(rn.is_pc())?;
                unpredictable(rt.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    rm,
                    shift: Shift::lsl(ins.imm2(4)),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (offset, _) = shift_c(proc[self.rm], self.shift, carry_in);
        let address = proc[self.rn].wrapping_add(offset);
        let data = proc[self.rt];
        proc.set_u32le_at(address, data)?;
        Ok(false)
    }

    fn name(&self) -> String {
        "str".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, [{}, {}{}]",
            self.rt,
            self.rn,
            self.rm,
            self.shift.arg_string()
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

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        let rt = proc[self.rt];
        let rt2 = proc[self.rt2];
        proc.set_u32le_at(address, rt)?;
        proc.set_u32le_at(address.wrapping_add(4), rt2)?;
        if self.wback {
            proc.registers.set(self.rn, offset_addr);
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
