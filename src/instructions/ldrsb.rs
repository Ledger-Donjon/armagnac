//! Implements LDRSB (Load Register Signed Byte) instruction.

use super::{other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
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
    /// Encoding.
    pub tn: usize,
}

impl Instruction for LdrsbImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V7M, V7EM, V8M],
                expression: "111110011001xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110010001xxxxxxxx1xxxxxxxxxxx",
            },
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
                    tn,
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
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let offset_addr = proc[self.rn].wrapping_add_or_sub(self.imm32, self.add);
        let addr = if self.index { offset_addr } else { rn };
        let data = ((proc.read_u8(addr)? as i8) as i32) as u32;
        proc.set(self.rt, data);
        if self.wback {
            proc.set(self.rn, offset_addr);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrsb".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 1)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, self.index, self.add, self.wback)
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
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V7EM, V8M],
            expression: "11111001x0011111xxxxxxxxxxxxxxxx",
        }]
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
        let data = ((proc.read_u8(address)? as i8) as i32) as u32;
        proc.set(self.rt, data);
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrsb".into()
    }

    fn qualifier(&self) -> Qualifier {
        // llvm-objdump add .w qualifier despite Arm Architecture Reference Manual doesn't.
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(RegisterIndex::Pc, self.imm32, false, true, self.add, false)
        )
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
    /// Encoding.
    tn: usize,
}

impl Instruction for LdrsbReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0101011xxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110010001xxxxxxxx000000xxxxxx",
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
                tn,
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
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        // From the specification, INDEX is always true, ADD is always true and WBACK always false,
        // so the implementation has been simplified.
        let (offset, _) = shift_c(proc[self.rm], self.shift, proc.registers.psr.c());
        let rn = proc[self.rn];
        let address = rn.wrapping_add(offset);
        let data = ((proc.read_u8(address)? as i8) as i32) as u32;
        proc.set(self.rt, data);
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrsb".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 2)
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
        arith::Shift,
        arm::{ArmProcessor, Config},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    #[test]
    fn test_ldrsb_imm() {
        let mut proc = ArmProcessor::new(Config::v7m());
        proc.map_ram(0x1000, 4).unwrap();
        proc.write_u32le_iface(0x1000, 0x12b456f8).unwrap();

        proc.registers.r1 = 0x1000 - 16;
        let mut ins = LdrsbImm {
            rn: RegisterIndex::R1,
            rt: RegisterIndex::R0,
            imm32: 16,
            index: true,
            add: true,
            wback: true,
            tn: 0,
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
        let mut proc = ArmProcessor::new(Config::v8m());
        proc.map_ram(0x1000, 4).unwrap();
        proc.write_u32le_iface(0x1000, 0x12b456f8).unwrap();

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
        let mut proc = ArmProcessor::new(Config::v8m());
        proc.map_ram(0x1000, 4).unwrap();
        proc.write_u32le_iface(0x1000, 0x12b456f8).unwrap();

        let mut ins = LdrsbReg {
            rt: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift: Shift::lsl(0),
            tn: 0,
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
