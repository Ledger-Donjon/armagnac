//! Implements STRH (Store Register Halfword) instruction.

use core::panic;

use crate::{
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{indexing_args, other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction};

/// STRH (immediate) instruction.
pub struct StrhImm {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset from Rn.
    imm32: u32,
    /// True to store with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// True to write new offset value back to Rn.
    wback: bool,
}

impl Instruction for StrhImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "10000xxxxxxxxxxx",
            "111110001010xxxxxxxxxxxxxxxxxxxx",
            "111110000010xxxxxxxx1xxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: ins.imm5(6) << 1,
                index: true,
                add: true,
                wback: false,
            },
            2 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                undefined(rt.is_pc())?;
                unpredictable(rt.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    imm32: ins.imm12(0),
                    index: true,
                    add: true,
                    wback: false,
                }
            }
            3 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                let (p, u, w) = ins.puw();
                other(p && u && !w)?; // STRHT
                undefined(rn.is_pc() || (!p && !w))?;
                unpredictable(rt.is_sp_or_pc() || (w && rt == rn))?;
                Self {
                    rt,
                    rn,
                    imm32: ins.imm8(0),
                    index: p,
                    add: u,
                    wback: w,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        proc.set_u16le_at(address, proc[self.rt] as u16)?;
        if self.wback {
            proc.set(self.rn, offset_addr)
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "strh".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, self.index, self.add, self.wback,)
        )
    }
}

/// STRH (register) instruction.
///
/// Store Register Halfword.
pub struct StrhReg {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Rm shift amount.
    shift: u8,
}

impl Instruction for StrhReg {
    fn patterns() -> &'static [&'static str] {
        &["0101001xxxxxxxxx", "111110000010xxxxxxxx000000xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: 0,
            },
            2 => {
                let rn = ins.reg4(16);
                undefined(rn.is_pc())?;
                let rt = ins.reg4(12);
                let rm = ins.reg4(0);
                unpredictable(rt.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    rm,
                    shift: ins.imm2(4) as u8,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let offset = shift_c(proc[self.rm], Shift::lsl(self.shift as u32), carry_in).0;
        let address = proc[self.rn].wrapping_add(offset);
        proc.set_u16le_at(address, proc[self.rt] as u16)?;
        Ok(false)
    }

    fn name(&self) -> String {
        "strh".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, [{}, {}{}]",
            self.rt,
            self.rn,
            self.rm,
            Shift::lsl(self.shift as u32).arg_string()
        )
    }
}
