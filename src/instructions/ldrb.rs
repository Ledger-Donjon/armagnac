//! Implements LDRB (Load Register Byte) instruction.

use super::{ldr::LdrImm, other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction};
use crate::{
    align::Align,
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    it_state::ItState,
    registers::RegisterIndex,
};
use core::panic;

/// LDRB (immediate) instruction.
///
/// Load Register Byte (immediate).
pub struct LdrbImm(LdrImm);

impl Instruction for LdrbImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "01111xxxxxxxxxxx",
            "111110001001xxxxxxxxxxxxxxxxxxxx",
            "111110000001xxxxxxxx1xxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self(LdrImm {
                rn: ins.reg3(3),
                rt: ins.reg3(0),
                imm32: ins.imm5(6),
                index: true,
                add: true,
                wback: false,
            }),
            2 => {
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                other(rt.is_pc())?; // PLD
                other(rn.is_pc())?; // LDRB (literal)
                unpredictable(rt.is_sp())?;
                Self(LdrImm {
                    rn,
                    rt,
                    imm32: ins.imm12(0),
                    index: true,
                    add: true,
                    wback: false,
                })
            }
            3 => {
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                let (p, u, w) = ins.puw();
                other(rt.is_pc() && p)?; // PLD
                other(rn.is_pc())?; // LDRB (literal)
                other(p && u && !w)?; // LDRBT
                undefined(!p && !w)?;
                unpredictable(rt.is_sp_or_pc() || (w && rt == rn))?;
                Self(LdrImm {
                    rn,
                    rt,
                    imm32: ins.imm8(0),
                    index: p,
                    add: u,
                    wback: w,
                })
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.0.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.0.imm32, self.0.add);
        let addr = if self.0.index { offset_addr } else { rn };
        let data = proc.u8_at(addr)?;
        proc.set(self.0.rt, data as u32);
        if self.0.wback {
            proc.set(self.0.rn, offset_addr);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrb".into()
    }

    fn args(&self, pc: u32) -> String {
        self.0.args(pc)
    }
}

/// LDRB (register) instruction.
///
/// Load Register Byte (register).
pub struct LdrbReg {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
}

impl Instruction for LdrbReg {
    fn patterns() -> &'static [&'static str] {
        &["0101110xxxxxxxxx", "111110000001xxxxxxxx000000xxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
            },
            2 => {
                let rm = ins.reg4(0);
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // LDR (literal)
                unpredictable(rm.is_sp_or_pc())?;
                unpredictable(rt.is_pc() && state.in_it_block_not_last())?;
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
        // From the specification, INDEX is always true, ADD is always true and WBACK always false,
        // so the implementation has been simplified.
        let (offset, _) = shift_c(proc[self.rm], self.shift, proc.registers.psr.c());
        let addr = proc[self.rn].wrapping_add(offset);
        let data = proc.u8_at(addr)?;
        proc.set(self.rt, data as u32);
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrb".into()
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

/// LDRB (literal) instruction.
///
/// Load Register Byte (literal).
pub struct LdrbLit {
    /// Destination register.
    rt: RegisterIndex,
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
}

impl Instruction for LdrbLit {
    fn patterns() -> &'static [&'static str] {
        &["11111000x0011111xxxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rt = ins.reg4(12);
        other(rt.is_pc())?; // PLD
        unpredictable(rt.is_sp())?;
        Ok(Self {
            rt,
            imm32: ins.imm12(0),
            add: ins.bit(23),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let base = proc.pc().align(4);
        let address = base.wrapping_add_or_sub(self.imm32, self.add);
        let data = proc.u8_at(address)?;
        proc.set(self.rt, data as u32);
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrb".into()
    }

    fn args(&self, pc: u32) -> String {
        let address = pc.wrapping_add(4).align(4).wrapping_add(self.imm32);
        format!("{}, [pc, #{}]  ; 0x{:0x}", self.rt, self.imm32, address)
    }
}
