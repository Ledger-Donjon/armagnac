//! Implements LDR (Load Register) instruction.

use super::{other, undefined, unpredictable, DecodeHelper, Instruction};
use crate::{
    align::Align,
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::AddOrSub,
    it_state::ItState,
    registers::RegisterIndex,
};
use core::panic;

pub struct LdrImm {
    /// Base register.
    pub rn: RegisterIndex,
    /// Destination register.
    pub rt: RegisterIndex,
    /// Offset from Rn.
    pub imm32: u32,
    /// True to load with indexing.
    pub index: bool,
    /// True to add offset, false to subtract.
    pub add: bool,
    /// True to write new offset value back to Rn.
    pub wback: bool,
}

impl Instruction for LdrImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "01101xxxxxxxxxxx",
            "10011xxxxxxxxxxx",
            "111110001101xxxxxxxxxxxxxxxxxxxx",
            "111110000101xxxxxxxx1xxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rn: ins.reg3(3),
                rt: ins.reg3(0),
                imm32: ((ins >> 6) & 0x1f) << 2,
                index: true,
                add: true,
                wback: false,
            },
            2 => Self {
                rn: RegisterIndex::Sp,
                rt: ins.reg3(8),
                imm32: (ins & 0xff) << 2,
                index: true,
                add: true,
                wback: false,
            },
            3 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                other(rn.is_pc())?; // LDR (literal)
                unpredictable(rt.is_pc() && state.in_it_block_not_last())?;
                Self {
                    rn,
                    rt,
                    imm32: ins & 0xfff,
                    index: true,
                    add: true,
                    wback: false,
                }
            }
            4 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                let puw = (ins >> 8) & 7;
                let imm8 = ins.imm8(0);
                let wback = puw & 1 != 0;
                other(rn.is_pc())?; // LDR (literal)
                other(puw == 6)?; // LDRT
                other(rn.is_sp() && puw == 3 && imm8 == 4)?; // POP
                undefined(puw & 5 == 0)?;
                unpredictable((wback && rn == rt) || (rt.is_pc() && state.in_it_block_not_last()))?;
                Self {
                    rn,
                    rt,
                    imm32: imm8,
                    index: puw & 4 != 0,
                    add: puw & 2 != 0,
                    wback,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let addr = if self.index { offset_addr } else { rn };
        let data = proc.u32le_at(addr)?;
        if self.wback {
            proc.set(self.rn, offset_addr);
        }
        if self.rt.is_pc() {
            if addr & 3 == 0 {
                todo!();
            } else {
                return Err(RunError::InstructionUnpredictable);
            }
        } else {
            proc.set(self.rt, data)
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldr".into()
    }

    fn args(&self, _pc: u32) -> String {
        match (self.index, self.wback) {
            (true, false) => {
                let imm = if self.imm32 != 0 {
                    format!(", #{}", self.imm32)
                } else {
                    "".into()
                };
                format!("{}, [{}{}]", self.rt, self.rn, imm)
            }
            (true, true) => format!("{}, [{}, #{}]!", self.rt, self.rn, self.imm32),
            (false, true) => format!("{}, [{}], #{}", self.rt, self.rn, self.imm32),
            (false, false) => panic!(), // Filtered out by try_decode
        }
    }
}

/// LDR (literal) instruction.
pub struct LdrLit {
    /// Destination register.
    rt: RegisterIndex,
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
}

impl Instruction for LdrLit {
    fn patterns() -> &'static [&'static str] {
        &["01001xxxxxxxxxxx", "11111000x1011111xxxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: ins.reg3(8),
                imm32: (ins & 0xff) << 2,
                add: true,
            },
            2 => {
                let rt = ins.reg4(12);
                unpredictable(rt.is_pc() && state.in_it_block_not_last())?;
                Self {
                    rt,
                    imm32: ins & 0xfff,
                    add: ins.bit(23),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let base = proc.pc().align(4);
        let addr = base.wrapping_add_or_sub(self.imm32, self.add);
        let data = proc.u32le_at(addr)?;
        if self.rt.is_pc() {
            if addr & 3 == 0 {
                todo!();
            } else {
                return Err(RunError::InstructionUnpredictable);
            }
        } else {
            proc.set(self.rt, data)
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldr".into()
    }

    fn args(&self, pc: u32) -> String {
        let address = pc.wrapping_add(4).align(4).wrapping_add(self.imm32);
        format!("{}, [pc, #{}]  ; 0x{:0x}", self.rt, self.imm32, address)
    }
}

/// LDR (register) instruction.
pub struct LdrReg {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift to be applied to Rn.
    shift: Shift,
    /// True to load with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// True to write new offset value back to Rn.
    wback: bool,
}

impl Instruction for LdrReg {
    fn patterns() -> &'static [&'static str] {
        &["0101100xxxxxxxxx", "111110000101xxxxxxxx000000xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
                index: true,
                add: true,
                wback: false,
            },
            2 => {
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                other(rn.is_pc())?; // LDR (literal)
                unpredictable(rm.is_sp_or_pc())?;
                unpredictable(rt.is_pc() && state.in_it_block_not_last())?;
                Self {
                    rt,
                    rn,
                    rm,
                    shift: Shift::lsl((ins >> 4) & 3),
                    index: true,
                    add: true,
                    wback: false,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let (offset, _) = shift_c(proc[self.rm], self.shift, proc.registers.xpsr.c());
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(offset, self.add);
        let address = if self.index { offset_addr } else { rn };
        let data = proc.u32le_at(address)?;
        if self.wback {
            proc.set(self.rn, offset_addr);
        }
        if self.rt.is_pc() {
            if address & 3 == 0 {
                proc.bx_write_pc(data)?;
                return Ok(true);
            } else {
                return Err(RunError::InstructionUnpredictable);
            }
        } else {
            proc.set(self.rt, data)
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldr".into()
    }

    fn args(&self, _pc: u32) -> String {
        if self.shift.n == 0 {
            format!("{}, [{}, {}]", self.rt, self.rn, self.rm)
        } else {
            format!(
                "{}, [{}, {}, {}]",
                self.rt,
                self.rn,
                self.rm,
                self.shift.arg_string()
            )
        }
    }
}
