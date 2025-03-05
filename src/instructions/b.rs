//! Implements B (Branch) instruction.

use crate::{
    arith::sign_extend, arm::ArmProcessor, condition::Condition, decoder::DecodeError,
    instructions::other, it_state::ItState,
};

use super::{undefined, unpredictable, Instruction};

/// B instruction.
pub struct B {
    /// Condition.
    cond: Option<Condition>,
    /// Offset.
    imm32: i32,
}

impl Instruction for B {
    fn patterns() -> &'static [&'static str] {
        &[
            "1101xxxxxxxxxxxx",
            "11100xxxxxxxxxxx",
            "11110xxxxxxxxxxx10x0xxxxxxxxxxxx",
            "11110xxxxxxxxxxx10x1xxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                // May match SVC if cond is 15
                let cond = Condition::try_from(ins >> 8 & 0xf).map_err(|_| DecodeError::Other)?;
                undefined(cond == Condition::Always)?;
                Self {
                    cond: Some(cond),
                    imm32: sign_extend((ins & 0xff) << 1, 9),
                }
            }
            2 => {
                unpredictable(state.in_it_block_not_last())?;
                Self {
                    cond: None,
                    imm32: sign_extend((ins & 0x7ff) << 1, 12),
                }
            }
            3 => {
                other(ins >> 23 & 7 == 7)?;
                // cond cannot be 15 because of the previous test, so the following conversion
                // cannot fail.
                let cond = Condition::try_from(ins >> 22 & 0x0f).unwrap();
                other(cond == Condition::Always)?; // Can be ISB for instance
                                                   // I think there is an error in ARM spec on J1 and J2, to be checked.
                let imm21 = (ins >> 26 & 1) << 20
                    | (ins >> 11 & 1) << 19
                    | (ins >> 13 & 1) << 18
                    | (ins >> 16 & 0x3f) << 12
                    | (ins & 0x7ff) << 1;
                let imm32 = sign_extend(imm21, 21);
                unpredictable(state.in_it_block())?;
                Self {
                    cond: Some(cond),
                    imm32,
                }
            }
            4 => {
                let s = ins >> 26 & 1;
                let i1 = 1 ^ (ins >> 13 & 1) ^ s;
                let i2 = 1 ^ (ins >> 11 & 1) ^ s;
                let imm25 =
                    s << 24 | i1 << 23 | i2 << 22 | (ins >> 16 & 0x3ff) << 12 | (ins & 0x7ff) << 1;
                let imm32 = sign_extend(imm25, 25);
                unpredictable(state.in_it_block_not_last())?;
                Self { cond: None, imm32 }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, crate::arm::RunError> {
        let address = (proc.pc() as i32 + self.imm32) as u32;
        proc.set_pc(address);
        Ok(true)
    }

    fn condition(&self) -> Option<Condition> {
        self.cond
    }

    fn name(&self) -> String {
        format!(
            "b{}",
            if let Some(cond) = self.cond {
                cond
            } else {
                Condition::Always
            }
        )
    }

    fn args(&self, pc: u32) -> String {
        // PC value of a Thumb instruction is it's address + 4
        let label = (pc as i32 + self.imm32) as u32 + 4;
        format!("0x{:x}", label)
    }
}
