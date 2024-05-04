use crate::{
    arm::{Arm7Processor, RunError},
    condition::Condition,
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

pub mod add;
pub mod and;
pub mod b;
pub mod bic;
pub mod bl;
pub mod blx;
pub mod bx;
pub mod cbnz;
pub mod clz;
pub mod cmp;
pub mod cps;
pub mod dsb;
pub mod isb;
pub mod it;
pub mod ldm;
pub mod ldr;
pub mod ldrb;
pub mod ldrh;
pub mod lsl;
pub mod lsr;
pub mod mla;
pub mod mov;
pub mod mrs;
pub mod msr;
pub mod mul;
pub mod mvn;
pub mod nop;
pub mod orr;
pub mod pop;
pub mod push;
pub mod rsb;
pub mod sev;
pub mod stmdb;
pub mod str;
pub mod strb;
pub mod strh;
pub mod sub;
pub mod ubfx;
pub mod udiv;
pub mod uxt;
pub mod tst;

/// All instructions must implement this trait in order to be integrated into the emulator.
pub trait Instruction {
    fn patterns() -> &'static [&'static str]
    where
        Self: Sized;
    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError>
    where
        Self: Sized;

    /// Returns instruction execution own condition, or None if the condition in the IT state is to
    /// be followed. Only the B instruction in T1 and T3 encodings should implement this, all other
    /// instructions rely on the blanket implementation which returns [None].
    fn condition(&self) -> Option<Condition> {
        None
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError>;

    fn name(&self) -> String;

    fn args(&self, pc: u32) -> String;
}

/// If condition passes, returns [DecodeError::Unpredictable] error.
pub fn unpredictable(cond: bool) -> Result<(), DecodeError> {
    if cond {
        Err(DecodeError::Unpredictable)
    } else {
        Ok(())
    }
}

/// If condition passes, returns [DecodeError::Undefined] error.
pub fn undefined(cond: bool) -> Result<(), DecodeError> {
    if cond {
        Err(DecodeError::Undefined)
    } else {
        Ok(())
    }
}

/// If condition passes, returns [DecodeError::Other] error.
pub fn other(cond: bool) -> Result<(), DecodeError> {
    if cond {
        Err(DecodeError::Other)
    } else {
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum InstructionSize {
    Ins16 = 2,
    Ins32 = 4,
}

pub trait DecodeHelper {
    fn reg3(&self, lsb_index: u8) -> RegisterIndex;
    fn reg4(&self, lsb_index: u8) -> RegisterIndex;
    fn imm1(&self, lsb_index: u8) -> u32;
    fn imm2(&self, lsb_index: u8) -> u32;
    fn imm3(&self, lsb_index: u8) -> u32;
    fn imm5(&self, lsb_index: u8) -> u32;
    fn imm8(&self, lsb_index: u8) -> u32;
    fn imm12(&self, lsb_index: u8) -> u32;
    fn puw(&self) -> (bool, bool, bool);
}

impl DecodeHelper for u32 {
    fn reg3(&self, lsb_index: u8) -> RegisterIndex {
        reg(self >> lsb_index & 0x7)
    }

    fn reg4(&self, lsb_index: u8) -> RegisterIndex {
        reg(self >> lsb_index & 0xf)
    }

    fn imm1(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 1
    }

    fn imm2(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 3
    }

    fn imm3(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 7
    }

    fn imm5(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 0x1f
    }

    fn imm8(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 0xff
    }

    fn imm12(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 0xfff
    }

    fn puw(&self) -> (bool, bool, bool) {
        (self & 1 << 10 != 0, self & 1 << 9 != 0, self & 1 << 8 != 0)
    }
}

/// Returns thumb instruction size based on first halfword value.
///
/// # Arguments
///
/// * `halfword` - First halfword of the instruction.
pub fn thumb_ins_size(halfword: u16) -> InstructionSize {
    match (halfword & 0xf800) >> 11 {
        0b11101..=0b11111 => InstructionSize::Ins32,
        _ => InstructionSize::Ins16,
    }
}

/// Helper function for instruction decoding to create a [RegisterIndex], giving easier
/// readability.
fn reg(index: u32) -> RegisterIndex {
    RegisterIndex::new_main(index)
}

/// Returns "{rd}" if rd is equal to rn, else "{rd}, {rn}".
///
/// This is a convenient method used for many instruction arguments formatting.
pub fn rdn_args_string(rd: RegisterIndex, rn: RegisterIndex) -> String {
    if rd == rn {
        format!("{rd}")
    } else {
        format!("{rd}, {rn}")
    }
}

pub fn indexing_args(rn: RegisterIndex, imm: u32, index: bool, add: bool, wback: bool) -> String {
    let neg = if add { "" } else { "-" };
    match (index, wback) {
        (true, false) => {
            let imm = if imm != 0 {
                format!(", #{}{}", neg, imm)
            } else {
                "".into()
            };
            format!("[{}{}]", rn, imm)
        }
        (true, true) => format!("[{rn}, #{neg}{imm}]!"),
        (false, true) => format!("[{rn}], #{neg}{imm}"),
        (false, false) => panic!(),
    }
}

/// Utility trait to call either addition or subtraction between two values depending on a
/// condition.
///
/// This is to be used by many instructions which have a bit indicating if an offset is to be added
/// to a base value, or subtracted.
pub trait AddOrSub {
    fn add_or_sub(&self, rhs: Self, add: bool) -> Self;
    fn wrapping_add_or_sub(&self, rhs: Self, add: bool) -> Self;
}

impl AddOrSub for u32 {
    fn add_or_sub(&self, rhs: Self, add: bool) -> Self {
        if add {
            self + rhs
        } else {
            self - rhs
        }
    }

    fn wrapping_add_or_sub(&self, rhs: Self, add: bool) -> Self {
        if add {
            self.wrapping_add(rhs)
        } else {
            self.wrapping_sub(rhs)
        }
    }
}
