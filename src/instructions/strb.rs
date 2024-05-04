//! Implements STRB (immediate) and STRB (register) instructions.

use crate::{
    arith::{shift_c, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::ItState,
    registers::RegisterIndex,
};

use super::{indexing_args, other, reg, undefined, unpredictable, AddOrSub, Instruction};

/// STRB (immediate) instruction.
pub struct StrbImm {
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

impl Instruction for StrbImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "01110xxxxxxxxxxx",
            "111110001000xxxxxxxxxxxxxxxxxxxx",
            "111110000000xxxxxxxx1xxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: reg(ins & 7),
                rn: reg(ins >> 3 & 7),
                imm32: ins >> 6 & 0x1f,
                index: true,
                add: true,
                wback: false,
            },
            2 => {
                let rn = reg(ins >> 16 & 0xf);
                let rt = reg(ins >> 12 & 0xf);
                undefined(rn.is_pc())?;
                unpredictable(rt.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    imm32: ins & 0xfff,
                    index: true,
                    add: true,
                    wback: false,
                }
            }
            3 => {
                let puw = ins >> 8 & 7;
                let rn = reg(ins >> 16 & 0xf);
                let rt = reg(ins >> 12 & 0xf);
                let wback = puw & 1 != 0;
                other(puw == 6)?; // STRBT
                undefined(rn.is_pc() || puw & 5 == 0)?;
                unpredictable(rt.is_sp_or_pc() || (wback && rn == rt))?;
                Self {
                    rt,
                    rn,
                    imm32: ins & 0xff,
                    index: puw & 4 != 0,
                    add: puw & 2 != 0,
                    wback: wback,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        proc.set_u8_at(address, (proc.registers[self.rt] & 0xff) as u8)?;
        if self.wback {
            proc.registers[self.rn] = offset_addr
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "strb".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, self.index, self.add, self.wback,)
        )
    }
}

/// STRB (register) instruction.
pub struct StrbReg {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Rm shift amount.
    shift: u8,
}

impl Instruction for StrbReg {
    fn patterns() -> &'static [&'static str] {
        &["0101010xxxxxxxxx", "111110000000xxxxxxxx000000xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: reg(ins & 7),
                rn: reg(ins >> 3 & 7),
                rm: reg(ins >> 6 & 7),
                shift: 0,
            },
            2 => {
                let rn = reg(ins >> 16 & 0xf);
                let rt = reg(ins >> 12 & 0xf);
                let rm = reg(ins & 0xf);
                undefined(rn.is_pc())?;
                unpredictable(rt.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    rm,
                    shift: (ins >> 4 & 3) as u8,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.apsr.c();
        let shift = Shift::lsl(self.shift as u32);
        let (offset, _) = shift_c(proc.registers[self.rm], shift, carry_in);
        let address = proc.registers[self.rn].wrapping_add(offset);
        proc.set_u8_at(address, (proc.registers[self.rt] & 0xff) as u8)?;
        Ok(false)
    }

    fn name(&self) -> String {
        "strb".into()
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
