//! Implements B (Branch) instruction.

use super::ArmVersion::{V6M, V7EM, V7M, V8M};
use super::Encoding::{self, T1, T2, T3, T4};
use super::{undefined, unpredictable, Instruction, Pattern, Qualifier};
use crate::core::{Effect, RunError};
use crate::qualifier_wide_match;
use crate::{
    arith::sign_extend, core::ArmProcessor, core::Condition, core::ItState, decoder::DecodeError,
    instructions::other,
};

/// B instruction.
pub struct B {
    /// Condition.
    cond: Option<Condition>,
    /// Offset.
    imm32: i32,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for B {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1101xxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "11100xxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11110xxxxxxxxxxx10x0xxxxxxxxxxxx",
            },
            Pattern {
                encoding: T4,
                versions: &[V7M, V7EM, V8M],
                expression: "11110xxxxxxxxxxx10x1xxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                // May match SVC if cond is 15
                let cond = Condition::try_from((ins >> 8) & 0xf).map_err(|_| DecodeError::Other)?;
                undefined(cond == Condition::Always)?;
                Self {
                    cond: Some(cond),
                    imm32: sign_extend((ins & 0xff) << 1, 9),
                    encoding,
                }
            }
            T2 => {
                unpredictable(state.in_it_block_not_last())?;
                Self {
                    cond: None,
                    imm32: sign_extend((ins & 0x7ff) << 1, 12),
                    encoding,
                }
            }
            T3 => {
                other((ins >> 23) & 7 == 7)?;
                // cond cannot be 15 because of the previous test, so the following conversion
                // cannot fail.
                let cond = Condition::try_from((ins >> 22) & 0x0f).unwrap();
                other(cond == Condition::Always)?; // Can be ISB for instance
                                                   // I think there is an error in ARM spec on J1 and J2, to be checked.
                let imm21 = (((ins >> 26) & 1) << 20)
                    | (((ins >> 11) & 1) << 19)
                    | (((ins >> 13) & 1) << 18)
                    | (((ins >> 16) & 0x3f) << 12)
                    | ((ins & 0x7ff) << 1);
                let imm32 = sign_extend(imm21, 21);
                unpredictable(state.in_it_block())?;
                Self {
                    cond: Some(cond),
                    imm32,
                    encoding,
                }
            }
            T4 => {
                let s = (ins >> 26) & 1;
                let i1 = 1 ^ ((ins >> 13) & 1) ^ s;
                let i2 = 1 ^ ((ins >> 11) & 1) ^ s;
                let imm25 = (s << 24)
                    | (i1 << 23)
                    | (i2 << 22)
                    | (((ins >> 16) & 0x3ff) << 12)
                    | ((ins & 0x7ff) << 1);
                let imm32 = sign_extend(imm25, 25);
                unpredictable(state.in_it_block_not_last())?;
                Self {
                    cond: None,
                    imm32,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let address = (proc.pc() as i32 + self.imm32) as u32;
        proc.set_pc(address);
        Ok(Effect::Branch)
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

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T3 | T4)
    }

    fn args(&self, pc: u32) -> String {
        // PC value of a Thumb instruction is it's address + 4
        let label = (pc as i32 + self.imm32) as u32 + 4;
        format!("0x{:x}", label)
    }
}
