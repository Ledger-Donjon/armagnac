use core::panic;
use std::{
    fmt::{self, Debug, Display},
    ops::{Index, IndexMut},
};

use crate::{condition::Condition, helpers::TestBit};

#[derive(Copy, Clone, Default)]
pub struct Register(pub u32);

impl Register {
    pub fn bit(&self, n: usize) -> bool {
        self.0.bit(n)
    }

    pub fn set_bit(&mut self, n: u8, value: bool) {
        self.0 = (self.0 & !(1 << n)) | (value as u32) << n
    }

    pub fn val(&self) -> u32 {
        self.0
    }

    pub fn set_val(&mut self, value: u32) {
        self.0 = value
    }
}

impl Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

/// Enumeration to identify a CPU core register
///
/// Provides methods to convert to/from instruction encoding values.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RegisterIndex {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    Sp,
    Lr,
    Pc,
    Apsr,
    Iapsr,
    Eapsr,
    Xpsr,
    Ipsr,
    Epsr,
    Iepsr,
    Msp,
    Psp,
    Primask,
    Basepri,
    BasepriMask,
    FaultMask,
    Control,
}

impl RegisterIndex {
    pub fn new_main(index: u32) -> Self {
        match index {
            0 => Self::R0,
            1 => Self::R1,
            2 => Self::R2,
            3 => Self::R3,
            4 => Self::R4,
            5 => Self::R5,
            6 => Self::R6,
            7 => Self::R7,
            8 => Self::R8,
            9 => Self::R9,
            10 => Self::R10,
            11 => Self::R11,
            12 => Self::R12,
            13 => Self::Sp,
            14 => Self::Lr,
            15 => Self::Pc,
            _ => panic!("invalid main register index"),
        }
    }

    pub fn new_sys(index: u32) -> Self {
        match index {
            0 => Self::Apsr,
            1 => Self::Iapsr,
            2 => Self::Eapsr,
            3 => Self::Xpsr,
            5 => Self::Ipsr,
            6 => Self::Epsr,
            7 => Self::Iepsr,
            8 => Self::Msp,
            9 => Self::Psp,
            16 => Self::Primask,
            17 => Self::Basepri,
            18 => Self::BasepriMask,
            19 => Self::FaultMask,
            20 => Self::Control,
            _ => panic!("invalid sys register index"),
        }
    }

    /// Index of the register if it is R0-R15, None otherwise
    pub fn index_main(&self) -> Option<u32> {
        match self {
            Self::R0 => Some(0),
            Self::R1 => Some(1),
            Self::R2 => Some(2),
            Self::R3 => Some(3),
            Self::R4 => Some(4),
            Self::R5 => Some(5),
            Self::R6 => Some(6),
            Self::R7 => Some(7),
            Self::R8 => Some(8),
            Self::R9 => Some(9),
            Self::R10 => Some(10),
            Self::R11 => Some(11),
            Self::R12 => Some(12),
            Self::Sp => Some(13),
            Self::Lr => Some(14),
            Self::Pc => Some(15),
            _ => None,
        }
    }

    /// Returns true if index is 13
    pub fn is_sp(&self) -> bool {
        *self == Self::Sp
    }

    /// Returns true if index is 15
    pub fn is_pc(&self) -> bool {
        *self == Self::Pc
    }

    /// Returns true if index is 13 or 15
    pub fn is_sp_or_pc(&self) -> bool {
        (*self == Self::Sp) || (*self == Self::Pc)
    }
}

impl From<u32> for RegisterIndex {
    fn from(value: u32) -> Self {
        Self::new_main(value)
    }
}

impl Display for RegisterIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::R0 => "r0",
            Self::R1 => "r1",
            Self::R2 => "r2",
            Self::R3 => "r3",
            Self::R4 => "r4",
            Self::R5 => "r5",
            Self::R6 => "r6",
            Self::R7 => "r7",
            Self::R8 => "r8",
            Self::R9 => "r9",
            Self::R10 => "r10",
            Self::R11 => "r11",
            Self::R12 => "r12",
            Self::Sp => "sp",
            Self::Lr => "lr",
            Self::Pc => "pc",
            Self::Apsr => "apsr",
            Self::Iapsr => "iapsr",
            Self::Eapsr => "eapsr",
            Self::Xpsr => "xpsr",
            Self::Ipsr => "ipsr",
            Self::Epsr => "epsr",
            Self::Iepsr => "iepsr",
            Self::Msp => "msp",
            Self::Psp => "psp",
            Self::Primask => "primask",
            Self::Basepri => "basepri",
            Self::BasepriMask => "baseprimask",
            Self::FaultMask => "faultmask",
            Self::Control => "control",
        };
        write!(f, "{}", s)
    }
}

/// List of R0-R15 registers used in some instructions encoding, such as PUSH.
///
/// Each bit of the data maps to a register.
#[derive(Debug, Copy, Clone)]
pub struct MainRegisterList(u16);

impl MainRegisterList {
    pub fn new(bits: u16) -> MainRegisterList {
        MainRegisterList(bits)
    }

    /// Returns `true` if the list contains no registers.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if given register is in the list.
    pub fn contains(&self, x: &RegisterIndex) -> bool {
        if let Some(i) = x.index_main() {
            self.0 & (1 << i) != 0
        } else {
            false
        }
    }

    /// Returns the number of registers in the list
    pub fn len(&self) -> usize {
        let mut result = 0;
        let mut x = self.0;
        while x > 0 {
            if x & 1 != 0 {
                result += 1;
            }
            x >>= 1;
        }
        result
    }

    /// Returns whether the list contains the SP register (13)
    pub fn has_sp(&self) -> bool {
        self.contains(&RegisterIndex::Sp)
    }

    /// Returns whether the list contains the LR register (14)
    pub fn has_lr(&self) -> bool {
        self.contains(&RegisterIndex::Lr)
    }

    /// Returns whether the list contains the PC register (15)
    pub fn has_pc(&self) -> bool {
        self.contains(&RegisterIndex::Pc)
    }

    pub fn iter(&self) -> MainRegisterListIterator {
        MainRegisterListIterator {
            list: self.0,
            forward: 0,
            backward: 15,
        }
    }
}

impl Display for MainRegisterList {
    /// Formats list of registers, to produce a string such as "r0, r1, sp".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for reg in self.iter() {
            if first {
                first = false;
            } else {
                f.write_str(", ")?;
            }
            f.write_str(&reg.to_string())?;
        }
        Ok(())
    }
}

pub struct MainRegisterListIterator {
    /// Marks the registers present in the list. Bit 0 for R0, bit 1 for R1, etc.
    list: u16,
    /// Index of next register to be tested and returned if present, in the forward iteration
    /// direction.
    forward: i8,
    /// Index of the next register to be tested and returned if present, in the backward iteration
    /// direction.
    backward: i8,
}

impl Iterator for MainRegisterListIterator {
    type Item = RegisterIndex;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.forward > self.backward {
                return None;
            } else {
                let pos = self.forward;
                self.forward += 1;
                if self.list & (1 << pos) != 0 {
                    return Some(RegisterIndex::new_main(pos as u32));
                }
            }
        }
    }
}

impl DoubleEndedIterator for MainRegisterListIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.backward < self.forward {
                return None;
            } else {
                let pos = self.backward;
                self.backward -= 1;
                if self.list & (1 << pos) != 0 {
                    return Some(RegisterIndex::new_main(pos as u32));
                }
            }
        }
    }
}

/// Application Program Status Register
#[derive(Debug)]
pub struct ApplicationProgramStatusRegister(Register);

impl ApplicationProgramStatusRegister {
    pub fn new() -> Self {
        ApplicationProgramStatusRegister(Register(0))
    }

    pub fn n(&self) -> bool {
        self.0.bit(31)
    }

    pub fn set_n(&mut self, n: bool) -> &mut Self {
        self.0.set_bit(31, n);
        self
    }

    pub fn z(&self) -> bool {
        self.0.bit(30)
    }

    pub fn set_z(&mut self, z: bool) -> &mut Self {
        self.0.set_bit(30, z);
        self
    }

    pub fn c(&self) -> bool {
        self.0.bit(29)
    }

    pub fn set_c(&mut self, c: bool) -> &mut Self {
        self.0.set_bit(29, c);
        self
    }

    /// Set the carry bit value, if defined.
    ///
    /// # Arguments
    ///
    /// * `c` - If some, set the carry bit to this value, otherwise does nothing.
    pub fn set_c_opt(&mut self, c: Option<bool>) -> &mut Self {
        if let Some(c) = c {
            self.set_c(c);
        }
        self
    }

    pub fn v(&self) -> bool {
        self.0.bit(28)
    }

    pub fn set_v(&mut self, v: bool) -> &mut Self {
        self.0.set_bit(28, v);
        self
    }

    pub fn q(&self) -> bool {
        self.0.bit(28)
    }

    pub fn set_q(&mut self, q: bool) -> &mut Self {
        self.0.set_bit(27, q);
        self
    }

    pub fn set_nz(&mut self, v: u32) -> &mut Self {
        self.set_n(v >> 31 != 0).set_z(v == 0)
    }

    /// Returns true if execution condition is met regarding current flags in APSR.
    ///
    /// # Arguments
    ///
    /// * `cond` - Tested condition
    pub fn test(&self, cond: Condition) -> bool {
        match cond {
            Condition::Equal => self.z(),
            Condition::NotEqual => !self.z(),
            Condition::CarrySet => self.c(),
            Condition::CarryClear => !self.c(),
            Condition::Minus => self.n(),
            Condition::Plus => !self.n(),
            Condition::Overflow => self.v(),
            Condition::NoOverflow => !self.v(),
            Condition::Higher => self.c() && !self.z(),
            Condition::LowerOrSame => !self.c() || self.z(),
            Condition::GreaterThanOrEqual => self.n() == self.v(),
            Condition::LessThan => self.n() != self.v(),
            Condition::GreaterThan => !self.z() && self.n() == self.v(),
            Condition::LessThanOrEqual => self.z() || self.n() != self.v(),
            Condition::Always => true,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExecutionProgramStatusRegister(Register);

impl ExecutionProgramStatusRegister {
    pub fn new() -> Self {
        Self(Register(0))
    }

    pub fn t(&self) -> bool {
        self.0.bit(24)
    }

    pub fn set_t(&mut self, value: bool) {
        self.0.set_bit(24, value)
    }
}

/// Base struct for PRIMASK and FAULTMASK registers
#[derive(Debug)]
pub struct MaskRegister(Register);

impl MaskRegister {
    pub fn new() -> Self {
        Self(Register(0))
    }

    /// Returns priority mask bit
    pub fn pm(&self) -> bool {
        self.0.bit(0)
    }

    /// Sets priority mask bit
    ///
    /// # Arguments
    ///
    /// * `pm` - Priority mask bit value
    pub fn set_pm(&mut self, pm: bool) {
        self.0.set_bit(0, pm)
    }
}

/// Special purpose control register
#[derive(Debug)]
pub struct ControlRegister(Register);

impl ControlRegister {
    /// Returns register state at reset.
    pub fn new() -> Self {
        Self(Register(0))
    }

    /// Returns false if thread has privileged mode, true if not.
    pub fn privileged_bit(&self) -> bool {
        self.0.bit(0)
    }

    /// Change the privileged thread bit
    ///
    /// # Arguments
    ///
    /// * `value` - false to enable privileged thread mode
    pub fn set_privileged_bit(&mut self, value: bool) {
        self.0.set_bit(0, value)
    }

    /// Returns false if MSP is used, true if PSP is used (in thread mode).
    pub fn stack_bit(&self) -> bool {
        self.0.bit(1)
    }

    /// Change the stack selection bit
    ///
    /// * `value` - false to select MSP, true to select PSP (in thread mode).
    pub fn set_stack_bit(&mut self, value: bool) {
        self.0.set_bit(1, value)
    }
}

#[derive(Debug)]
pub struct CoreRegisters {
    pub r0: Register,
    pub r1: Register,
    pub r2: Register,
    pub r3: Register,
    pub r4: Register,
    pub r5: Register,
    pub r6: Register,
    pub r7: Register,
    pub r8: Register,
    pub r9: Register,
    pub r10: Register,
    pub r11: Register,
    pub r12: Register,
    pub lr: Register,
    pub pc: Register,
    pub msp: Register,
    pub psp: Register,
    pub apsr: ApplicationProgramStatusRegister,
    pub epsr: ExecutionProgramStatusRegister,
    pub primask: MaskRegister,
    pub faultmask: MaskRegister,
    pub control: ControlRegister,
    /// Current execution mode
    /// Used in particular to return MSP or PSP when SP is requested
    pub handler_mode: bool,
}

impl CoreRegisters {
    pub fn new() -> Self {
        Self {
            r0: Register(0),
            r1: Register(0),
            r2: Register(0),
            r3: Register(0),
            r4: Register(0),
            r5: Register(0),
            r6: Register(0),
            r7: Register(0),
            r8: Register(0),
            r9: Register(0),
            r10: Register(0),
            r11: Register(0),
            r12: Register(0),
            lr: Register(0),
            pc: Register(0),
            msp: Register(0),
            psp: Register(0),
            apsr: ApplicationProgramStatusRegister::new(),
            epsr: ExecutionProgramStatusRegister::new(),
            primask: MaskRegister::new(),
            faultmask: MaskRegister::new(),
            control: ControlRegister::new(),
            handler_mode: false,
        }
    }

    /// Returns `RegisterIndex::Msp` or `RegisterIndex::Psp` depending on current execution mode
    /// and control register value.
    pub fn translate_sp(&self) -> RegisterIndex {
        if self.handler_mode {
            RegisterIndex::Msp
        } else {
            if self.control.stack_bit() {
                RegisterIndex::Psp
            } else {
                RegisterIndex::Msp
            }
        }
    }

    pub fn sp(&self) -> &Register {
        &self[self.translate_sp()]
    }

    pub fn sp_mut(&mut self) -> &mut Register {
        let sp = self.translate_sp();
        &mut self[sp]
    }
}

impl Index<u32> for CoreRegisters {
    type Output = Register;

    fn index(&self, index: u32) -> &Self::Output {
        &self[RegisterIndex::new_main(index)]
    }
}

impl IndexMut<u32> for CoreRegisters {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self[RegisterIndex::new_main(index)]
    }
}

impl Index<RegisterIndex> for CoreRegisters {
    type Output = Register;

    fn index(&self, index: RegisterIndex) -> &Self::Output {
        match index {
            RegisterIndex::R0 => &self.r0,
            RegisterIndex::R1 => &self.r1,
            RegisterIndex::R2 => &self.r2,
            RegisterIndex::R3 => &self.r3,
            RegisterIndex::R4 => &self.r4,
            RegisterIndex::R5 => &self.r5,
            RegisterIndex::R6 => &self.r6,
            RegisterIndex::R7 => &self.r7,
            RegisterIndex::R8 => &self.r8,
            RegisterIndex::R9 => &self.r9,
            RegisterIndex::R10 => &self.r10,
            RegisterIndex::R11 => &self.r11,
            RegisterIndex::R12 => &self.r12,
            RegisterIndex::Sp => match self.translate_sp() {
                RegisterIndex::Msp => &self.msp,
                RegisterIndex::Psp => &self.psp,
                _ => panic!(),
            },
            RegisterIndex::Lr => &self.lr,
            RegisterIndex::Pc => &self.pc,
            RegisterIndex::Apsr => &self.apsr.0,
            RegisterIndex::Iapsr => todo!(),
            RegisterIndex::Eapsr => todo!(),
            RegisterIndex::Xpsr => todo!(),
            RegisterIndex::Ipsr => todo!(),
            RegisterIndex::Epsr => todo!(),
            RegisterIndex::Iepsr => todo!(),
            RegisterIndex::Msp => &self.msp,
            RegisterIndex::Psp => &self.psp,
            RegisterIndex::Primask => &self.primask.0,
            RegisterIndex::Basepri => todo!(),
            RegisterIndex::BasepriMask => todo!(),
            RegisterIndex::FaultMask => &self.faultmask.0,
            RegisterIndex::Control => &self.control.0,
        }
    }
}

impl IndexMut<RegisterIndex> for CoreRegisters {
    fn index_mut(&mut self, index: RegisterIndex) -> &mut Self::Output {
        match index {
            RegisterIndex::R0 => &mut self.r0,
            RegisterIndex::R1 => &mut self.r1,
            RegisterIndex::R2 => &mut self.r2,
            RegisterIndex::R3 => &mut self.r3,
            RegisterIndex::R4 => &mut self.r4,
            RegisterIndex::R5 => &mut self.r5,
            RegisterIndex::R6 => &mut self.r6,
            RegisterIndex::R7 => &mut self.r7,
            RegisterIndex::R8 => &mut self.r8,
            RegisterIndex::R9 => &mut self.r9,
            RegisterIndex::R10 => &mut self.r10,
            RegisterIndex::R11 => &mut self.r11,
            RegisterIndex::R12 => &mut self.r12,
            RegisterIndex::Sp => match self.translate_sp() {
                RegisterIndex::Msp => &mut self.msp,
                RegisterIndex::Psp => &mut self.psp,
                _ => panic!(),
            },
            RegisterIndex::Lr => &mut self.lr,
            RegisterIndex::Pc => &mut self.pc,
            RegisterIndex::Apsr => &mut self.apsr.0,
            RegisterIndex::Iapsr => todo!(),
            RegisterIndex::Eapsr => todo!(),
            RegisterIndex::Xpsr => todo!(),
            RegisterIndex::Ipsr => todo!(),
            RegisterIndex::Epsr => todo!(),
            RegisterIndex::Iepsr => todo!(),
            RegisterIndex::Msp => &mut self.msp,
            RegisterIndex::Psp => &mut self.psp,
            RegisterIndex::Primask => &mut self.primask.0,
            RegisterIndex::Basepri => todo!(),
            RegisterIndex::BasepriMask => todo!(),
            RegisterIndex::FaultMask => &mut self.faultmask.0,
            RegisterIndex::Control => &mut self.control.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::registers::RegisterIndex;

    use super::MainRegisterList;

    #[test]
    fn test_main_register_list_iterator() {
        // Test forward iteration
        assert!(MainRegisterList(0xaaaa)
            .iter()
            .eq((1..16).step_by(2).map(|i| RegisterIndex::new_main(i))));
        assert!(MainRegisterList(0x5555)
            .iter()
            .eq((0..16).step_by(2).map(|i| RegisterIndex::new_main(i))));
        assert!(MainRegisterList(0xff00)
            .iter()
            .eq((8..16).map(|i| RegisterIndex::new_main(i))));
        assert!(MainRegisterList(0x00ff)
            .iter()
            .eq((0..8).map(|i| RegisterIndex::new_main(i))));
        // Test backward iteration
        assert!(MainRegisterList(0xaaaa)
            .iter()
            .rev()
            .eq((1..16).step_by(2).rev().map(|i| RegisterIndex::new_main(i))));
        assert!(MainRegisterList(0x5555)
            .iter()
            .rev()
            .eq((0..16).step_by(2).rev().map(|i| RegisterIndex::new_main(i))));
        assert!(MainRegisterList(0xff00)
            .iter()
            .rev()
            .eq((8..16).rev().map(|i| RegisterIndex::new_main(i))));
        assert!(MainRegisterList(0x00ff)
            .iter()
            .rev()
            .eq((0..8).rev().map(|i| RegisterIndex::new_main(i))));
    }
}
