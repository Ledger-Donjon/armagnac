//! Implements STR (Store Register) instruction.

use super::{indexing_args, other, undefined, unpredictable, AddOrSub, Instruction};
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
use crate::{
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{DecodeHelper, ItState},
    registers::RegisterIndex,
};

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
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "01100xxxxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V6M, V7M, V8M],
                expression: "10010xxxxxxxxxxx",
            },
            Pattern {
                tn: 3,
                versions: &[V7M, V8M],
                expression: "111110001100xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                tn: 4,
                versions: &[V7M, V8M],
                expression: "111110000100xxxxxxxx1xxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: ((ins >> 6) & 0x1f) << 2,
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
                let puw = (ins >> 8) & 7;
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
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        proc.write_u32_unaligned(address, proc[self.rt])?;
        if self.wback {
            proc.set(self.rn, offset_addr)
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
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "0101000xxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "111110000100xxxxxxxx000000xxxxxx",
            },
        ]
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
        let carry_in = proc.registers.psr.c();
        let (offset, _) = shift_c(proc[self.rm], self.shift, carry_in);
        let address = proc[self.rn].wrapping_add(offset);
        let data = proc[self.rt];
        proc.write_u32_unaligned(address, data)?;
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
