//! Defines ARM processor core registers.

use crate::{core::Condition, core::ItState, helpers::BitAccess, instructions::DecodeHelper};
use core::panic;
use std::{
    fmt::{self, Debug, Display},
    ops::Index,
};

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
    BasepriMax,
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

    /// Generates a random register index from R0 to R12 (general purpose registers).
    /// This is used by tests only.
    #[cfg(test)]
    pub fn new_general_random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        Self::new_main(rng.random_range(..=12))
    }

    /// Generate two distinct random register indexes from R0 to R12 (general purpose registers).
    /// This is used by tests only.
    #[cfg(test)]
    pub fn pick_two_general_distinct() -> (Self, Self) {
        use rand::Rng;
        let mut rng = rand::rng();
        let range = 0..=12;
        let first = rng.random_range(range.clone());
        let mut second = rng.random_range(range.clone());
        while second == first {
            second = rng.random_range(range.clone());
        }
        (Self::new_main(first), Self::new_main(second))
    }

    /// Generate four distinct random register indexes from R0 to R12 (general purpose registers).
    /// This is used by tests only.
    #[cfg(test)]
    pub fn pick_four_general_distinct() -> (Self, Self, Self, Self) {
        let mut rng = rand::rng();
        let regs: Vec<_> = rand::seq::index::sample(&mut rng, 13, 4)
            .into_iter()
            .map(|i| Self::new_main(i as u32))
            .collect();
        (regs[0], regs[1], regs[2], regs[3])
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
            18 => Self::BasepriMax,
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

    pub fn index_sys(&self) -> u32 {
        match self {
            RegisterIndex::Apsr => 0,
            RegisterIndex::Iapsr => 1,
            RegisterIndex::Eapsr => 2,
            RegisterIndex::Xpsr => 3,
            RegisterIndex::Ipsr => 5,
            RegisterIndex::Epsr => 6,
            RegisterIndex::Iepsr => 7,
            RegisterIndex::Msp => 8,
            RegisterIndex::Psp => 9,
            RegisterIndex::Primask => 16,
            RegisterIndex::Basepri => 17,
            RegisterIndex::BasepriMax => 18,
            RegisterIndex::FaultMask => 19,
            RegisterIndex::Control => 20,
            _ => panic!("not a sys register"),
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

    /// Returns `true` if register is a low register (R0 to R7).
    pub fn is_low(&self) -> bool {
        matches!(
            self,
            Self::R0 | Self::R1 | Self::R2 | Self::R3 | Self::R4 | Self::R5 | Self::R6 | Self::R7
        )
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
            Self::BasepriMax => "basepri_max",
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

    /// Iterates over the registers present in the list.
    pub fn iter(&self) -> MainRegisterListIterator {
        MainRegisterListIterator {
            list: self.0,
            forward: 0,
            backward: 15,
        }
    }

    /// Returns the lowest register in the list, or `None` if the list is empty.
    pub fn lowest(&self) -> Option<RegisterIndex> {
        self.iter().next()
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

/// Program Status Register (PSR).
///
/// Regroups APSR, IPSR and EPSR together.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProgramStatusRegister(u32);

impl ProgramStatusRegister {
    /// APSR sub register bits mask.
    const APSR_MASK: u32 = 0xf80f0000;
    /// IPSR sub register bits mask.
    const IPSR_MASK: u32 = 0x000001ff;
    /// EPSR sub register bits mask.
    const EPSR_MASK: u32 = 0x0700fc00;

    pub fn new() -> Self {
        ProgramStatusRegister(0)
    }

    /// Returns XPSR register value.
    pub fn get(&self) -> u32 {
        self.0
    }

    /// Sets register value, except reserved and unused bits.
    pub fn set(&mut self, value: u32) {
        self.0 = value & 0xff0ffdff;
    }

    /// Returns APSR register value.
    pub fn apsr(&self) -> u32 {
        self.0 & Self::APSR_MASK
    }

    /// Sets APSR register value.
    pub fn set_apsr(&mut self, value: u32) {
        self.0 = self.0 & !Self::APSR_MASK | value & Self::APSR_MASK;
    }

    /// Returns IPSR register value.
    pub fn ipsr(&self) -> u32 {
        self.0 & Self::IPSR_MASK
    }

    /// Sets IPSR register value.
    pub fn set_ipsr(&mut self, value: u32) {
        self.0 = self.0 & !Self::IPSR_MASK | value & Self::IPSR_MASK;
    }

    /// Returns EPSR register value.
    pub fn epsr(&self) -> u32 {
        self.0 & Self::EPSR_MASK
    }

    /// Sets EPSR register value.
    pub fn set_epsr(&mut self, value: u32) {
        self.0 = self.0 & !Self::EPSR_MASK | value & Self::EPSR_MASK;
    }

    /// Returns Negative condition flag.
    pub fn n(&self) -> bool {
        self.0.bit(31)
    }

    /// Sets Negative condition flag.
    pub fn set_n(&mut self, n: bool) -> &mut Self {
        self.0.set_bit(31, n);
        self
    }

    /// Returns Zero condition flag.
    pub fn z(&self) -> bool {
        self.0.bit(30)
    }

    /// Sets Zero condition flag.
    pub fn set_z(&mut self, z: bool) -> &mut Self {
        self.0.set_bit(30, z);
        self
    }

    /// Returns Carry condition flag.
    pub fn c(&self) -> bool {
        self.0.bit(29)
    }

    /// Sets Carry condition flag.
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

    /// Returns Overflow condition flag.
    pub fn v(&self) -> bool {
        self.0.bit(28)
    }

    /// Sets Overflow condition flag.
    pub fn set_v(&mut self, v: bool) -> &mut Self {
        self.0.set_bit(28, v);
        self
    }

    /// Returns Saturation result bit.
    pub fn q(&self) -> bool {
        self.0.bit(27)
    }

    /// Sets Saturation result bit.
    pub fn set_q(&mut self, q: bool) -> &mut Self {
        self.0.set_bit(27, q);
        self
    }

    /// Sets N (negative) and Z (zero) flags from a value.
    ///
    /// ```
    /// # use armagnac::registers::ProgramStatusRegister;
    /// let mut psr = ProgramStatusRegister::new();
    /// psr.set_nz(0x80000000);
    /// assert!(psr.n() && !psr.z());
    /// psr.set_nz(0);
    /// assert!(!psr.n() && psr.z());
    /// ```
    pub fn set_nz(&mut self, v: u32) -> &mut Self {
        self.set_n(v >> 31 != 0).set_z(v == 0)
    }

    /// Returns all flags field from the APSR register at once (N, Z, C, V and Q).
    ///
    /// ```
    /// # use armagnac::registers::ProgramStatusRegister;
    /// let mut psr = ProgramStatusRegister::new();
    /// psr.set_n(true).set_c(true);
    /// assert_eq!(psr.flags(), 0b10100);
    /// ```
    pub fn flags(&self) -> u8 {
        (self.0 >> 27) as u8
    }

    /// Set all flags at once (N, Z, C, V and Q bits).
    ///
    /// ```
    /// # use armagnac::registers::ProgramStatusRegister;
    /// let mut psr = ProgramStatusRegister::new();
    /// // Set N and C flags, clear Z, V and Q.
    /// psr.set_flags(0b10100);
    /// assert!(psr.n() && psr.c());
    /// ```
    pub fn set_flags(&mut self, flags: u8) {
        debug_assert!(flags < 32);
        self.0 = self.0 & 0x07ffffff | ((flags as u32) << 27);
    }

    /// Returns Greater than or Equal flags (DSP extension).
    pub fn ge(&self) -> u8 {
        self.0.imm4(16) as u8
    }

    /// Sets Greater than or Equal flags (DSP extension).
    pub fn set_ge(&mut self, value: u8) {
        debug_assert!(value <= 0xf);
        self.0 = (self.0 & 0x000f0000) | ((value as u32 & 0xf) << 16);
    }

    pub fn ici_it(&self) -> u8 {
        (((self.0 & 0x06000000) >> 19) | ((self.0 >> 10) & 0x3f)) as u8
    }

    pub fn set_ici_it(&mut self, value: u8) -> &mut Self {
        let value = value as u32;
        self.0 = self.0 & 0xf9ff03ff | ((value & 0xc0) << 19) | ((value & 0x3f) << 10);
        self
    }

    pub fn t(&self) -> bool {
        self.0.bit(24)
    }

    pub fn set_t(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(24, value);
        self
    }

    /// Returns the exception number of the IPSR register.
    pub fn exception_number(&self) -> u16 {
        (self.0 & 0x1ff) as u16
    }

    /// Sets the exception number in the IPSR register.
    ///
    /// # Arguments
    ///
    /// * `value` - Exception number, on 9 bits.
    pub fn set_exception_number(&mut self, number: u16) -> &mut Self {
        debug_assert!(number <= 0x1ff);
        self.set_ipsr(number as u32);
        self
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

    /// Returns an [`ItState`] from the EPSR register value.
    pub fn it_state(&self) -> ItState {
        ItState((((self.0 >> 25) & 3) | (((self.0 >> 10) & 0x3f) << 2)) as u8)
    }

    /// Sets IT state in the EPSR register from an [`ItState`] object.
    pub fn set_it_state(&mut self, it: ItState) {
        self.0 = (self.0 & 0xf9ff03ff) | (((it.0 & 3) as u32) << 25) | (((it.0 >> 2) as u32) << 10);
    }
}

impl Default for ProgramStatusRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for ProgramStatusRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

/// Base struct for PRIMASK and FAULTMASK registers
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaskRegister(u32);

impl MaskRegister {
    pub fn new() -> Self {
        Self(0)
    }

    /// Returns priority mask bit (bit 0).
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

impl Default for MaskRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for MaskRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

/// Special purpose control register
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ControlRegister(u32);

impl ControlRegister {
    /// Returns register state at reset.
    pub fn new() -> Self {
        Self(0)
    }

    /// Returns raw value of the register.
    pub fn read(&self) -> u32 {
        self.0
    }

    /// Returns false if thread has privileged mode, true if not.
    pub fn privileged_bit(&self) -> bool {
        self.0.bit(0)
    }

    /// Change the privileged thread bit.
    ///
    /// # Arguments
    ///
    /// * `value` - false to enable privileged thread mode.
    pub fn set_privileged_bit(&mut self, value: bool) {
        self.0.set_bit(0, value)
    }

    /// Returns false if MSP is used, true if PSP is used (in thread mode).
    pub fn spsel(&self) -> bool {
        self.0.bit(1)
    }

    /// Change the stack selection bit
    ///
    /// * `value` - false to select MSP, true to select PSP (in thread mode).
    pub fn set_spsel(&mut self, value: bool) {
        self.0.set_bit(1, value)
    }
}

impl Default for ControlRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for ControlRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

/// Processor execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Fundamental mode for application execution. Selected on reset.
    Thread,
    /// Mode for exceptions execution.
    Handler,
}

/// Core registers of the ARM processor.
///
/// Registers can be accessed directly, e.g.:
///
/// ```
/// # use armagnac::core::{Processor, Config};
/// let mut proc = Processor::new(Config::v7m());
/// proc.registers.r0 = 0x1234;
/// ```
///
/// Or using indexing (read-only):
///
/// ```
/// # use armagnac::core::{Processor, Config};
/// # use armagnac::registers::RegisterIndex;
/// # let proc = Processor::new(Config::v7m());
/// let index = RegisterIndex::R0;
/// let value = proc.registers[index];
/// // Or shorter form:
/// let value = proc[index];
/// ```
///
/// It is not possible to get a mutable reference to a register using indexing because some
/// registers share bits. For instance `APSR`, `IPSR` and `EPSR` are sub-registers of the `PSR`
/// register. Therefore, modifying a register using an index is done by calling
/// [CoreRegisters::set]:
///
/// ```
/// # use armagnac::core::{Processor, Config};
/// # use armagnac::registers::RegisterIndex;
/// # let mut proc = Processor::new(Config::v7m());
/// let index = RegisterIndex::R0;
/// proc.registers.set(index, 0x1234);
/// // Or shorter form:
/// proc.set(index, 0x1234);
/// ```
///
/// Some registers have their bits and fields accessible with methods. This is the case for instance
/// of the program status register (or flags registers):
///
/// ```
/// # use armagnac::core::{Processor, Config};
/// # let mut proc = Processor::new(Config::v7m());
/// if proc.registers.psr.c() {
///     println!("Carry flag is set!")
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreRegisters {
    /// Argument, result or scratch register 1.
    pub r0: u32,
    /// Argument, result or scratch register 2.
    pub r1: u32,
    /// Argument, result or scratch register 3.
    pub r2: u32,
    /// Argument, result or scratch register 4.
    pub r3: u32,
    /// Variable register 1.
    /// Also known as `v1`.
    pub r4: u32,
    /// Variable register 2.
    /// Also known as `v2`.
    pub r5: u32,
    /// Variable register 3.
    /// Also known as `v3`.
    pub r6: u32,
    /// Variable register 4.
    /// Also known as `v4`.
    pub r7: u32,
    /// ARM-state variable register 5.
    /// Also known as `v5`.
    pub r8: u32,
    /// ARM-state variable register 6.
    /// Static base in RWPI variants.
    /// Also known as `v6` and `sb`.
    pub r9: u32,
    /// ARM-state variable register 7.
    /// Stack limit pointer in stack-checked variants.
    /// Also known as `v7` and `sl`.
    pub r10: u32,
    /// ARM-state variable register 8.
    /// Also known as `v8`.
    pub r11: u32,
    /// Intra-procedure-call scratch register.
    /// Also known as `ip`.
    pub r12: u32,
    /// Link register.
    /// Also known as r14.
    pub lr: u32,
    /// Program counter.
    /// Also known as r15.
    pub pc: u32,
    /// Main stack pointer.
    /// Also known as `r13`.
    pub msp: u32,
    /// Process stack pointer.
    /// Also known as `r13`.
    pub psp: u32,
    /// Groups APSR, IPSR and EPSR registers.
    pub psr: ProgramStatusRegister,
    /// Exception mask register.
    pub primask: MaskRegister,
    /// Fault mask register.
    pub faultmask: MaskRegister,
    /// Special purpose control register.
    pub control: ControlRegister,
    /// Current execution mode.
    /// Used in particular to return MSP or PSP when SP is requested.
    pub mode: Mode,
    /// Event register.
    pub event: bool,
}

impl CoreRegisters {
    pub fn new() -> Self {
        Self {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            lr: 0,
            pc: 0,
            msp: 0,
            psp: 0,
            psr: ProgramStatusRegister::new(),
            primask: MaskRegister::new(),
            faultmask: MaskRegister::new(),
            control: ControlRegister::new(),
            mode: Mode::Thread,
            event: false,
        }
    }

    /// Returns [`RegisterIndex::Msp`] or [`RegisterIndex::Psp`] depending on current execution mode
    /// and control register value.
    pub fn translate_sp(&self) -> RegisterIndex {
        match self.mode {
            Mode::Handler => RegisterIndex::Msp,
            Mode::Thread => {
                if self.control.spsel() {
                    RegisterIndex::Psp
                } else {
                    RegisterIndex::Msp
                }
            }
        }
    }

    pub fn sp(&self) -> u32 {
        self[self.translate_sp()]
    }

    pub fn sp_mut(&mut self) -> &mut u32 {
        match self.translate_sp() {
            RegisterIndex::Msp => &mut self.msp,
            RegisterIndex::Psp => &mut self.psp,
            _ => panic!(),
        }
    }

    /// Sets a register value
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the register to be modified.
    /// * `value` - New register value.
    pub fn set(&mut self, index: RegisterIndex, value: u32) {
        match index {
            RegisterIndex::R0 => self.r0 = value,
            RegisterIndex::R1 => self.r1 = value,
            RegisterIndex::R2 => self.r2 = value,
            RegisterIndex::R3 => self.r3 = value,
            RegisterIndex::R4 => self.r4 = value,
            RegisterIndex::R5 => self.r5 = value,
            RegisterIndex::R6 => self.r6 = value,
            RegisterIndex::R7 => self.r7 = value,
            RegisterIndex::R8 => self.r8 = value,
            RegisterIndex::R9 => self.r9 = value,
            RegisterIndex::R10 => self.r10 = value,
            RegisterIndex::R11 => self.r11 = value,
            RegisterIndex::R12 => self.r12 = value,
            RegisterIndex::Sp => match self.translate_sp() {
                RegisterIndex::Msp => self.msp = value,
                RegisterIndex::Psp => self.psp = value,
                _ => panic!(),
            },
            RegisterIndex::Lr => self.lr = value,
            RegisterIndex::Pc => self.pc = value,
            RegisterIndex::Apsr => self.psr.set_apsr(value),
            RegisterIndex::Iapsr => todo!(),
            RegisterIndex::Eapsr => todo!(),
            RegisterIndex::Xpsr => self.psr.set(value),
            RegisterIndex::Ipsr => self.psr.set_ipsr(value),
            RegisterIndex::Epsr => self.psr.set_epsr(value),
            RegisterIndex::Iepsr => todo!(),
            RegisterIndex::Msp => self.msp = value,
            RegisterIndex::Psp => self.psp = value,
            RegisterIndex::Primask => self.primask.0 = value,
            RegisterIndex::Basepri => todo!(),
            RegisterIndex::BasepriMax => todo!(),
            RegisterIndex::FaultMask => self.faultmask.0 = value,
            RegisterIndex::Control => self.control.0 = value,
        }
    }
}

impl Default for CoreRegisters {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u32> for CoreRegisters {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        &self[RegisterIndex::new_main(index)]
    }
}

impl Index<RegisterIndex> for CoreRegisters {
    type Output = u32;

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
            RegisterIndex::Apsr => &self.psr.0,
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
            RegisterIndex::BasepriMax => todo!(),
            RegisterIndex::FaultMask => &self.faultmask.0,
            RegisterIndex::Control => &self.control.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MainRegisterList, ProgramStatusRegister};
    use crate::registers::RegisterIndex;

    #[test]
    fn test_register_index_is_low() {
        for i in 0..=15 {
            assert_eq!(RegisterIndex::new_main(i).is_low(), i <= 7);
        }
        for i in (0..=3).chain(5..=9).chain(16..=20) {
            assert!(!RegisterIndex::new_sys(i).is_low());
        }
    }

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

    #[test]
    fn test_program_status_register_flags() {
        // Test flags and set_flags
        let mut psr = ProgramStatusRegister::new();
        for i in 0..32 {
            psr.set_flags(i);
            assert_eq!(psr.flags(), i);
            assert_eq!(psr.n(), i & 0b10000 != 0);
            assert_eq!(psr.z(), i & 0b01000 != 0);
            assert_eq!(psr.c(), i & 0b00100 != 0);
            assert_eq!(psr.v(), i & 0b00010 != 0);
            assert_eq!(psr.q(), i & 0b00001 != 0);
            psr.set_flags(0);
            psr.set_n(i & 0b10000 != 0);
            psr.set_z(i & 0b01000 != 0);
            psr.set_c(i & 0b00100 != 0);
            psr.set_v(i & 0b00010 != 0);
            psr.set_q(i & 0b00001 != 0);
            assert_eq!(psr.flags(), i);
        }
    }
}
