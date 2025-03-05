//! Implements LDRSB (Load Register Signed Byte) instruction.

use crate::{
    align::Align,
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::indexing_args,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction};

/// LDRSB (immediate) instruction.
///
/// Load Register Signed Byte.
pub struct LdrsbImm {
    /// Destination register.
    pub rt: RegisterIndex,
    /// Base register.
    pub rn: RegisterIndex,
    /// Offset from Rn.
    pub imm32: u32,
    /// True to load with indexing.
    pub index: bool,
    /// True to add offset, false to subtract.
    pub add: bool,
    /// True to write new offset value back to Rn.
    pub wback: bool,
}

impl Instruction for LdrsbImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "111110011001xxxxxxxxxxxxxxxxxxxx",
            "111110010001xxxxxxxx1xxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rt = ins.reg4(12);
                other(rt.is_pc())?; // PLI
                unpredictable(rt.is_sp())?;
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // LDRSB (literal)
                Self {
                    rn,
                    rt,
                    imm32: ins.imm12(0),
                    index: true,
                    add: true,
                    wback: false,
                }
            }
            2 => {
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                let (p, u, w) = ins.puw();
                other(rt.is_pc() && p && !u && !w)?; // PLI
                other(rn.is_pc())?; // LDRSB (literal)
                other(p && u && !w)?; // LDRSBT
                undefined(!p && !w)?;
                unpredictable(rt.is_sp() || (w && rn == rt))?;
                unpredictable(rt.is_pc() && (!p || u || w))?;
                Self {
                    rn,
                    rt,
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
        let rn = proc.registers[self.rn];
        let offset_addr = proc.registers[self.rn].wrapping_add_or_sub(self.imm32, self.add);
        let addr = if self.index { offset_addr } else { rn };
        let data = ((proc.u8_at(addr)? as i8) as i32) as u32;
        proc.registers.set(self.rt, data);
        if self.wback {
            proc.registers.set(self.rn, offset_addr);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrsb".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, self.index, self.add, self.wback)
        )
    }
}

/// LDRSB (literal) instruction.
///
/// Load Register Signed Byte.
pub struct LdrsbLit {
    /// Destination register.
    rt: RegisterIndex,
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
}

impl Instruction for LdrsbLit {
    fn patterns() -> &'static [&'static str] {
        &["11111001x0011111xxxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        assert_eq!(tn, 1);
        let rt = ins.reg4(12);
        other(rt.is_pc())?; // PLI
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
        let data = ((proc.u8_at(address)? as i8) as i32) as u32;
        proc.registers.set(self.rt, data);
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrsb".into()
    }

    fn args(&self, pc: u32) -> String {
        let address = pc.wrapping_add(4).align(4).wrapping_add(self.imm32 as u32);
        format!("{}, [pc, #{}]  ; 0x{:0x}", self.rt, self.imm32, address)
    }
}

/// LDRSB (register) instruction.
///
/// Load Register Signed Byte.
pub struct LdrsbReg {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
}

impl Instruction for LdrsbReg {
    fn patterns() -> &'static [&'static str] {
        &["0101011xxxxxxxxx", "111110010001xxxxxxxx000000xxxxxx"]
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
                let rt = ins.reg4(12);
                other(rt.is_pc())?; // PLD
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // LDRB (literal)
                let rm = ins.reg4(0);
                unpredictable(rt.is_sp() || rm.is_sp_or_pc())?;
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
        let (offset, _) = shift_c(proc.registers[self.rm], self.shift, proc.registers.xpsr.c());
        let rn = proc.registers[self.rn];
        let address = rn.wrapping_add(offset);
        let data = ((proc.u8_at(address)? as i8) as i32) as u32;
        proc.registers.set(self.rt, data as u32);
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrsb".into()
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

#[cfg(test)]
mod tests {
    use super::{LdrsbImm, LdrsbLit, LdrsbReg};
    use crate::{
        arith::Shift, arm::ArmProcessor, instructions::Instruction, registers::RegisterIndex,
    };

    #[test]
    fn test_ldrsb_imm() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.map_ram(0x1000, 4).unwrap();
        proc.set_u32le_at(0x1000, 0x12b456f8).unwrap();

        proc.registers.r1 = 0x1000 - 16;
        let mut ins = LdrsbImm {
            rn: RegisterIndex::R1,
            rt: RegisterIndex::R0,
            imm32: 16,
            index: true,
            add: true,
            wback: true,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffffff8);
        assert_eq!(proc.registers.r1, 0x1000);

        proc.registers.r0 = 0;
        proc.registers.r1 = 0x1000 + 16;
        ins.add = false;
        ins.wback = false;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffffff8);
        assert_eq!(proc.registers.r1, 0x1000 + 16);

        proc.registers.r0 = 0;
        proc.registers.r1 = 0x1000;
        ins.index = false;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffffff8);
    }

    #[test]
    fn test_ldrsb_lit() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.map_ram(0x1000, 4).unwrap();
        proc.set_u32le_at(0x1000, 0x12b456f8).unwrap();

        let ins = LdrsbLit {
            rt: RegisterIndex::R0,
            imm32: 128,
            add: true,
        };

        proc.set_pc(0x1000 - 128);
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffffff8);
    }

    #[test]
    fn test_ldrsb_reg() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.map_ram(0x1000, 4).unwrap();
        proc.set_u32le_at(0x1000, 0x12b456f8).unwrap();

        let mut ins = LdrsbReg {
            rt: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift: Shift::lsl(0),
        };

        proc.registers.r1 = 0x1000;
        proc.registers.r2 = 0;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xfffffff8);
        proc.registers.r2 = 1;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0x56);
        ins.shift = Shift::lsl(1);
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffffb4);
    }
}
