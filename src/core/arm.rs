//! Implements ARM processor core functions and execution.
//!
//! [Processor] structure shall be used to define a system and emulate its processor and
//! attached peripherals.

use crate::{
    align::Align,
    core::{exclusive_monitor::LocalMonitor, Condition, Config, Coprocessor, Irq, MonitorState},
    decoder::{BasicInstructionDecoder, InstructionDecode, InstructionDecodeError},
    helpers::BitAccess,
    instructions::{Instruction, InstructionSize},
    memory::{Env, MemoryAccessError, MemoryInterface, MemoryOpAction, RamMemory},
    mpu::{v7m::MpuV7M, v8m::MemoryProtectionUnitV8M},
    registers::{CoreRegisters, Mode, RegisterIndex},
    system_control::SystemControl,
};
use core::panic;
use std::{
    cell::RefCell,
    collections::BTreeSet,
    ops::{Index, Range},
    rc::Rc,
};

struct MemoryMap {
    address: u32,
    size: u32,
    iface: Rc<RefCell<dyn MemoryInterface>>,
}

/// Errors that may happen during emulation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RunError {
    /// Instruction is unknown or not supported.
    InstructionUnknown,
    /// Instruction is unpredictable.
    InstructionUnpredictable,
    /// Instruction is undefined.
    InstructionUndefined,
    /// Execution leads to unpredictable result.
    Unpredictable,
    /// Error when reading the memory or a peripheral.
    MemRead {
        address: u32,
        size: u32,
        cause: MemoryAccessError,
    },
    /// Error when writing to the memory or a peripheral.
    MemWrite {
        address: u32,
        size: u32,
        value: u32,
        cause: MemoryAccessError,
    },
}

impl From<InstructionDecodeError> for RunError {
    fn from(e: InstructionDecodeError) -> Self {
        match e {
            InstructionDecodeError::Unknown => RunError::InstructionUnknown,
            InstructionDecodeError::Unpredictable => RunError::InstructionUnpredictable,
            InstructionDecodeError::Undefined => todo!(),
        }
    }
}

struct CodeHook {
    range: Range<usize>,
}

/// Possible events returned during processor execution.
#[derive(Clone)]
pub enum Event {
    Hook {
        address: u32,
    },
    /// Successful emulation of an instruction.
    Instruction {
        ins: InstructionBox,
    },
    /// CPU reset.
    Reset,
    Break(u8),
    DebugHint(u8),
}

struct MemoryMappings(Vec<MemoryMap>);

impl MemoryMappings {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn get(&self, address: u32) -> Option<&MemoryMap> {
        for mapping in self.0.iter() {
            let start = mapping.address as usize;
            let end = mapping.address as usize + mapping.size as usize;
            if ((address as usize) >= start) && ((address as usize) < end) {
                return Some(mapping);
            }
        }
        None
    }

    fn get_mut(&mut self, address: u32) -> Option<&mut MemoryMap> {
        for mapping in self.0.iter_mut() {
            let start = mapping.address as usize;
            let end = mapping.address as usize + mapping.size as usize;
            if ((address as usize) >= start) && ((address as usize) < end) {
                return Some(mapping);
            }
        }
        None
    }
}

/// ARM architecture version.
///
/// Used to specify which architecture is being emulated.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArmVersion {
    /// ARMv6-M
    ///
    /// Implemented by:
    /// - Cortex-M0
    /// - Cortex-M0+
    /// - Cortex-M1
    /// - SecurCore SC000
    V6M,
    /// ARMv7-M
    ///
    /// Implemented by:
    /// - Cortex-M3
    /// - SecurCore SC300
    V7M,
    /// ARMv7E-M
    ///
    /// Architecture built on ARMv7-M with DSP extension.
    ///
    /// Implemented by:
    /// - Cortex-M4
    /// - Cortex-M7
    V7EM,
    /// ARMv8-M
    ///
    /// Implemented by:
    /// - Cortex-M23
    /// - Cortex-M33
    /// - Cortex-M35P
    V8M,
}

/// ARM processor state and attached peripherals.
///
/// This is the main struct of Armagnac. Once instanciated, memories and peripherals can be mapped
/// in the memory space used by the processor. The processor can then be executed, inspected and
/// eventually modified on the fly.
///
/// The following example will execute a tiny assembly program:
///
/// ```
/// # use armagnac::core::{Processor, Config,  RunOptions, Emulator};
/// let mut proc = Processor::new(Config::v7m());
///
/// // Load a tiny assembly program at address 0x1000.
/// // This creates a RAM memory of 6 bytes.
/// // mov r0, #5
/// // mov r1, #2
/// // sub r2, r0, r1
/// proc.map(0x1000, &[0x05, 0x20, 0x02, 0x21, 0x42, 0x1a]);
///
/// proc.set_pc(0x1000);
/// // Run and limit execution to 3 instructions
/// proc.run(RunOptions::new().gas(3)).unwrap();
/// assert_eq!(proc.registers.r2, 3);
/// ```
///
/// By default [Processor] uses [BasicInstructionDecoder] for decoding instructions. When
/// running long emulations, you may want to use a more performant decoder, at the cost of a longer
/// initialization time:
///
/// ```
/// use armagnac::core::{Processor, Config, ArmVersion};
/// use armagnac::decoder::{Lut16AndGrouped32InstructionDecoder};
///
/// let mut proc = Processor::new(Config::v7m());
/// proc.instruction_decoder = Box::new(Lut16AndGrouped32InstructionDecoder::new(ArmVersion::V7M));
/// ```
///
/// You can also write your own decoder by implementing the [InstructionDecode] trait. For example,
/// when emulating a small function for fuzzing, you may only keep the instructions used in the
/// assembly of that particular method to make the instruction decoding faster.
pub struct Processor {
    /// ARM emutaled version.
    pub version: ArmVersion,
    /// r0-r15 and sys registers.
    pub registers: CoreRegisters,
    pub execution_priority: i16,
    /// Indicates which exceptions are currently active.
    exception_active: Vec<bool>,
    /// Indicates current execution state (running, waiting for event, ...)
    state: State,
    memory_mappings: MemoryMappings,
    /// The local monitor tags a memory address for exclusive accesses.
    pub local_monitor: LocalMonitor,
    /// Parses word or double-word values to decode them as executable ARM instructions.
    /// Since this is a performance critical task of the emulator, different implementation with
    /// different optimisation strategies, which may depend on the context, may be selected.
    pub instruction_decoder: Box<dyn InstructionDecode>,
    /// Number of elapsed CPU clock cycles.
    /// Although all instructions will take one clock cycle in Armagnac, this value can be
    /// different from the number of executed instruction in the case WFE (Wait For Event) or WFI
    /// (Wait For Interrupt) instructions are used.
    pub cycles: u64,
    code_hooks: Vec<CodeHook>,
    /// Read and write access in peripherals may trigger actions. For instance, writing to a
    /// special register may perform a software reset. Those special actions from peripherals are
    /// stacked in this attribute during instruction emulation, and then processed one the
    /// instruction is completed.
    memory_op_actions: Vec<MemoryOpAction>,
    /// Pending interrupt requests
    interrupt_requests: BTreeSet<Irq>,
    /// System control registers peripheral.
    /// Needed for instance to fetch VTOR during an exception.
    system_control: Rc<RefCell<SystemControl>>,
    /// Coprocessors.
    /// If Arm profile does not support coprocessors, this vector remains empty.
    pub coprocessors: Vec<Option<Rc<RefCell<dyn Coprocessor>>>>,
    /// When poping PC from the stack (during exception return for instance), PC should be aligned,
    /// otherwise execution is unpredictable according to the ARM specification. However, some
    /// implementation may still set LSB to 1 for Thumb mode, and this can work on some hardware.
    /// So to allow emulation in that case, `tolerate_pop_stack_unaligned_pc` can be set to `true`.
    /// If `false` (the default) an error will be reported by the emulation if PC is unaligned.
    pub tolerate_pop_stack_unaligned_pc: bool,
    /// Stacked events from emulation.
    events: Vec<Event>,
}

type InstructionBox = Rc<dyn Instruction>;

impl Processor {
    /// Creates a new Arm processor.
    ///
    /// A [`Config`] is passed for initialization.
    ///
    /// If the Arm architecture version is not defined in the configuration, this method panics.
    ///
    /// ```
    /// # use armagnac::core::{Processor, Config};
    /// let processor = Processor::new(Config::v7m());
    /// ```
    pub fn new(config: Config) -> Self {
        let version = config.version;
        let exception_count = 16usize.checked_add(config.external_exceptions).unwrap();
        let coprocessor_count = match version {
            ArmVersion::V6M => 0,
            ArmVersion::V7M | ArmVersion::V7EM | ArmVersion::V8M => 16,
        };
        let system_control = Rc::new(RefCell::new(SystemControl::new()));

        let mut processor = Self {
            version,
            registers: CoreRegisters::new(),
            state: State::Running,
            memory_mappings: MemoryMappings::new(),
            local_monitor: LocalMonitor::new(config.exclusives_reservation_granule),
            execution_priority: 0,
            exception_active: (0..exception_count).map(|_| false).collect(),
            instruction_decoder: Box::new(BasicInstructionDecoder::new(version)),
            cycles: 0,
            code_hooks: Vec::new(),
            memory_op_actions: Vec::new(),
            interrupt_requests: BTreeSet::new(),
            system_control: system_control.clone(),
            coprocessors: (0..coprocessor_count).map(|_| None).collect(),
            tolerate_pop_stack_unaligned_pc: false,
            events: Vec::new(),
        };

        processor.map_iface(0xe000e000, system_control).unwrap();
        match processor.version {
            ArmVersion::V6M => {}
            ArmVersion::V7M | ArmVersion::V7EM => processor
                .map_iface(0xe000ed90, Rc::new(RefCell::new(MpuV7M::new())))
                .unwrap(),
            ArmVersion::V8M => {
                processor
                    .map_iface(
                        0xe000ed90,
                        Rc::new(RefCell::new(MemoryProtectionUnitV8M::new(16))),
                    )
                    .unwrap();
            }
        }
        processor
    }

    /// Maps and returns a RAM memory with `data` as initial value. The size of the created memory
    /// is the size of `data`.
    pub fn map(
        &mut self,
        address: u32,
        data: &[u8],
    ) -> Result<Rc<RefCell<RamMemory>>, MapConflict> {
        let ram = Rc::new(RefCell::new(RamMemory::new_from_slice(data)));
        self.map_iface(address, ram.clone())?;
        Ok(ram)
    }

    /// Maps an interface to the memory space. Returns an error if the new interface size overflows
    /// the address space, or if the new interface memory region intersects with an already mapped
    /// memory region.
    pub fn map_iface(
        &mut self,
        address: u32,
        iface: Rc<RefCell<dyn MemoryInterface>>,
    ) -> Result<(), MapConflict> {
        let size = iface.borrow().size();

        // Check for size overflow
        if address.checked_add(size).is_none() {
            return Err(MapConflict);
        }

        // Check overlap with another mapping
        if self.memory_mappings.0.iter().any(|m| {
            let a = m.address.max(address);
            let b = (m.address + m.size).min(address + size);
            a < b
        }) {
            return Err(MapConflict);
        }

        self.memory_mappings.0.push(MemoryMap {
            address,
            size,
            iface,
        });
        Ok(())
    }

    /// Creates a new RAM memory and map it in the address space.
    ///
    /// RAM is initialized at 0.
    ///
    /// # Arguments
    ///
    /// * `address` - RAM memory start address.
    /// * `size` - RAM memory size.
    pub fn map_ram(
        &mut self,
        address: u32,
        size: u32,
    ) -> Result<Rc<RefCell<RamMemory>>, MapConflict> {
        let ram = Rc::new(RefCell::new(RamMemory::new_zero(size as usize)));
        self.map_iface(address, ram.clone())?;
        Ok(ram)
    }

    /// Defines the implementation of one of the 16 possible coprocessors.
    ///
    /// Panics if index is out of range or Arm version does not support coprocessors.
    pub fn set_coprocessor(&mut self, index: usize, coprocessor: Rc<RefCell<dyn Coprocessor>>) {
        assert!(index < 16);
        self.coprocessors[index] = Some(coprocessor)
    }

    pub fn hook_code(&mut self, range: Range<usize>) {
        self.code_hooks.push(CodeHook { range })
    }

    /// If given `address` is not aligned to `size`, set `UNALIGNED` bit in CFSR register and take
    /// usage fault exception. This method is used by memory access calls performed by
    /// instructions.
    fn usage_fault_if_unaligned(&mut self, address: u32, size: usize) -> Result<(), RunError> {
        if !address.is_aligned(size) {
            self.system_control.borrow_mut().cfsr.set_unaligned(true);
            self.exception_taken(Irq::UsageFault)?;
        }
        Ok(())
    }

    /// Implements `MemA_with_priv` and `MemU_with_priv` from Arm Architecture Reference Manual,
    /// for 8 bit read accesses.
    pub fn read_u8_with_priv(&mut self, address: u32, privileged: bool) -> Result<u8, RunError> {
        self.validate_address(address, privileged, false, false);
        self.read_u8_iface(address)
    }

    /// Implements `MemA_with_priv` and `MemU_with_priv` from Arm Architecture Reference Manual,
    /// for 8 bit write accesses.
    pub fn write_u8_with_priv(
        &mut self,
        address: u32,
        value: u8,
        privileged: bool,
    ) -> Result<(), RunError> {
        self.validate_address(address, privileged, false, false);
        self.write_u8_iface(address, value)
    }

    /// Implements `MemA_with_priv` from Arm Architecture Reference Manual, for 16 bit read
    /// accesses.
    pub fn read_u16_aligned_with_priv(
        &mut self,
        address: u32,
        privileged: bool,
    ) -> Result<u16, RunError> {
        self.usage_fault_if_unaligned(address, 2)?;
        self.validate_address(address, privileged, false, false);
        let mut value = self.read_u16le_iface(address)?;
        if self.system_control.borrow_mut().aircr.endianess() {
            value = value.swap_bytes()
        }
        Ok(value)
    }

    /// Implements `MemA_with_priv` from Arm Architecture Reference Manual, for 16 bit write
    /// accesses.
    pub fn write_u16_aligned_with_priv(
        &mut self,
        address: u32,
        mut value: u16,
        privileged: bool,
    ) -> Result<(), RunError> {
        self.usage_fault_if_unaligned(address, 2)?;
        self.validate_address(address, privileged, false, false);
        if self.system_control.borrow_mut().aircr.endianess() {
            value = value.swap_bytes()
        }
        self.write_u16le_iface(address, value)
    }

    /// Implements `MemA_with_priv` from Arm Architecture Reference Manual, for 32 bit read
    /// accesses.
    pub fn read_u32_aligned_with_priv(
        &mut self,
        address: u32,
        privileged: bool,
    ) -> Result<u32, RunError> {
        self.usage_fault_if_unaligned(address, 4)?;
        self.validate_address(address, privileged, false, false);
        let mut value = self.read_u32le_iface(address)?;
        if self.system_control.borrow_mut().aircr.endianess() {
            value = value.swap_bytes()
        }
        Ok(value)
    }

    /// Implements `MemA_with_priv` from Arm Architecture Reference Manual, for 32 bit write
    /// accesses.
    pub fn write_u32_aligned_with_priv(
        &mut self,
        address: u32,
        mut value: u32,
        privileged: bool,
    ) -> Result<(), RunError> {
        self.usage_fault_if_unaligned(address, 4)?;
        self.validate_address(address, privileged, false, false);
        if self.system_control.borrow_mut().aircr.endianess() {
            value = value.swap_bytes()
        }
        self.write_u32le_iface(address, value)
    }

    /// Implements `MemA` and `MemU` from Arm Architecture Reference Manual, for 8 bit read
    /// accesses.
    pub fn read_u8(&mut self, address: u32) -> Result<u8, RunError> {
        self.read_u8_with_priv(address, self.is_privileged())
    }

    /// Implements `MemA` and `MemU` from Arm Architecture Reference Manual, for 8 bit write
    /// accesses.
    pub fn write_u8(&mut self, address: u32, value: u8) -> Result<(), RunError> {
        self.write_u8_with_priv(address, value, self.is_privileged())
    }

    /// Implements `MemA` from Arm Architecture Reference Manual, for 16 bit read accesses.
    pub fn read_u16_aligned(&mut self, address: u32) -> Result<u16, RunError> {
        self.read_u16_aligned_with_priv(address, self.is_privileged())
    }

    /// Implements `MemA` from Arm Architecture Reference Manual, for 8 bit write accesses.
    pub fn write_u16_aligned(&mut self, address: u32, value: u16) -> Result<(), RunError> {
        self.write_u16_aligned_with_priv(address, value, self.is_privileged())
    }

    /// Implements `MemA` from Arm Architecture Reference Manual, for 32 bit read accesses.
    pub fn read_u32_aligned(&mut self, address: u32) -> Result<u32, RunError> {
        self.read_u32_aligned_with_priv(address, self.is_privileged())
    }

    /// Implements `MemA` from Arm Architecture Reference Manual, for 32 bit write accesses.
    pub fn write_u32_aligned(&mut self, address: u32, value: u32) -> Result<(), RunError> {
        self.write_u32_aligned_with_priv(address, value, self.is_privileged())
    }

    /// Implements `MemU_with_priv` from Arm Architecture Reference Manual, for 16 bit read
    /// accesses.
    pub fn read_u16_unaligned_with_priv(
        &mut self,
        address: u32,
        privileged: bool,
    ) -> Result<u16, RunError> {
        if address.is_aligned(2) {
            self.read_u16_aligned_with_priv(address, privileged)
        } else if self.system_control.borrow().ccr.unalign_trp() {
            self.system_control.borrow_mut().cfsr.set_unaligned(true);
            self.exception_taken(Irq::UsageFault)?;
            Ok(0)
        } else {
            // Unaligned access
            let v0 = self.read_u8_with_priv(address, privileged)?;
            let v1 = self.read_u8_with_priv(address.wrapping_add(1), privileged)?;
            if self.system_control.borrow().aircr.endianess() {
                Ok(v1 as u16 | ((v0 as u16) << 8))
            } else {
                Ok(v0 as u16 | ((v1 as u16) << 8))
            }
        }
    }

    /// Implements `MemU_with_priv` from Arm Architecture Reference Manual, for 16 bit write
    /// accesses.
    pub fn write_u16_unaligned_with_priv(
        &mut self,
        address: u32,
        value: u16,
        privileged: bool,
    ) -> Result<(), RunError> {
        if address.is_aligned(2) {
            self.write_u16_aligned_with_priv(address, value, privileged)
        } else if self.system_control.borrow().ccr.unalign_trp() {
            self.system_control.borrow_mut().cfsr.set_unaligned(true);
            self.exception_taken(Irq::UsageFault)?;
            Ok(())
        } else {
            // Unaligned access
            let v0 = value as u8;
            let v1 = (value >> 8) as u8;
            if self.system_control.borrow().aircr.endianess() {
                self.write_u8_with_priv(address, v1, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(1), v0, privileged)?;
            } else {
                self.write_u8_with_priv(address, v0, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(1), v1, privileged)?;
            }
            Ok(())
        }
    }

    /// Implements `MemU_with_priv` from Arm Architecture Reference Manual, for 32 bit read
    /// accesses.
    pub fn read_u32_unaligned_with_priv(
        &mut self,
        address: u32,
        privileged: bool,
    ) -> Result<u32, RunError> {
        if address.is_aligned(4) {
            self.read_u32_aligned_with_priv(address, privileged)
        } else if self.system_control.borrow().ccr.unalign_trp() {
            self.system_control.borrow_mut().cfsr.set_unaligned(true);
            self.exception_taken(Irq::UsageFault)?;
            Ok(0)
        } else {
            // Unaligned access
            let v0 = self.read_u8_with_priv(address, privileged)?;
            let v1 = self.read_u8_with_priv(address.wrapping_add(1), privileged)?;
            let v2 = self.read_u8_with_priv(address.wrapping_add(2), privileged)?;
            let v3 = self.read_u8_with_priv(address.wrapping_add(3), privileged)?;
            if self.system_control.borrow().aircr.endianess() {
                Ok(v3 as u32 | ((v2 as u32) << 8) | ((v1 as u32) << 16) | ((v0 as u32) << 24))
            } else {
                Ok(v0 as u32 | ((v1 as u32) << 8) | ((v2 as u32) << 16) | ((v3 as u32) << 24))
            }
        }
    }

    /// Implements `MemU_with_priv` from Arm Architecture Reference Manual, for 32 bit write
    /// accesses.
    pub fn write_u32_unaligned_with_priv(
        &mut self,
        address: u32,
        value: u32,
        privileged: bool,
    ) -> Result<(), RunError> {
        if address.is_aligned(4) {
            self.write_u32_aligned_with_priv(address, value, privileged)
        } else if self.system_control.borrow().ccr.unalign_trp() {
            self.system_control.borrow_mut().cfsr.set_unaligned(true);
            self.exception_taken(Irq::UsageFault)?;
            Ok(())
        } else {
            // Unaligned access
            let v0 = value as u8;
            let v1 = (value >> 8) as u8;
            let v2 = (value >> 16) as u8;
            let v3 = (value >> 24) as u8;
            if self.system_control.borrow().aircr.endianess() {
                self.write_u8_with_priv(address, v3, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(1), v2, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(2), v1, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(3), v0, privileged)?;
            } else {
                self.write_u8_with_priv(address, v0, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(1), v1, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(2), v2, privileged)?;
                self.write_u8_with_priv(address.wrapping_add(3), v3, privileged)?;
            }
            Ok(())
        }
    }

    /// Implements `MemU` from Arm Architecture Reference Manual, for 16 bit read accesses.
    pub fn read_u16_unaligned(&mut self, address: u32) -> Result<u16, RunError> {
        self.read_u16_unaligned_with_priv(address, self.is_privileged())
    }

    /// Implements `MemU` from Arm Architecture Reference Manual, for 16 bit write accesses.
    pub fn write_u16_unaligned(&mut self, address: u32, value: u16) -> Result<(), RunError> {
        self.write_u16_unaligned_with_priv(address, value, self.is_privileged())
    }

    /// Implements `MemU` from Arm Architecture Reference Manual, for 32 bit read accesses.
    pub fn read_u32_unaligned(&mut self, address: u32) -> Result<u32, RunError> {
        self.read_u32_unaligned_with_priv(address, self.is_privileged())
    }

    /// Implements `MemU` from Arm Architecture Reference Manual, for 32 bit write accesses.
    pub fn write_u32_unaligned(&mut self, address: u32, value: u32) -> Result<(), RunError> {
        self.write_u32_unaligned_with_priv(address, value, self.is_privileged())
    }

    /// Reads a byte at `address` without checking for privileges.
    pub fn read_u8_iface(&mut self, address: u32) -> Result<u8, RunError> {
        let mut env = Env::new(self.cycles, self.is_privileged());
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemRead {
                address,
                size: 1,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let read = mapping
            .iface
            .borrow_mut()
            .read_u8(address - mapping.address, &mut env);
        self.memory_op_actions.extend(env.actions);
        match read {
            Ok(val) => Ok(val),
            Err(e) => Err(RunError::MemRead {
                address,
                size: 1,
                cause: e,
            }),
        }
    }

    /// Write byte `value` at `address` without checking for privileges.
    pub fn write_u8_iface(&mut self, address: u32, value: u8) -> Result<(), RunError> {
        let mut env = Env::new(self.cycles, self.is_privileged());
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemWrite {
                address,
                size: 1,
                value: value as u32,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let write = mapping
            .iface
            .borrow_mut()
            .write_u8(address - mapping.address, value, &mut env);
        self.memory_op_actions.extend(env.actions);
        match write {
            Ok(()) => Ok(()),
            Err(e) => Err(RunError::MemWrite {
                address,
                size: 1,
                value: value as u32,
                cause: e,
            }),
        }
    }

    /// Write halfword `value` at `address` without checking for privileges or alignment.
    pub fn write_u16le_iface(&mut self, address: u32, value: u16) -> Result<(), RunError> {
        let mut env = Env::new(self.cycles, self.is_privileged());
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemWrite {
                address,
                size: 2,
                value: value as u32,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let write =
            mapping
                .iface
                .borrow_mut()
                .write_u16le(address - mapping.address, value, &mut env);
        self.memory_op_actions.extend(env.actions);
        match write {
            Ok(()) => Ok(()),
            Err(e) => Err(RunError::MemWrite {
                address,
                size: 2,
                value: value as u32,
                cause: e,
            }),
        }
    }

    /// Read halfword at `address` without checking for privileges or alignment.
    pub fn read_u16le_iface(&mut self, address: u32) -> Result<u16, RunError> {
        let mut env = Env::new(self.cycles, self.is_privileged());
        if let Some(mapping) = self.memory_mappings.get_mut(address) {
            let read = mapping
                .iface
                .borrow_mut()
                .read_u16le(address - mapping.address, &mut env);
            self.memory_op_actions.extend(env.actions);
            match read {
                Ok(val) => Ok(val),
                Err(e) => Err(RunError::MemRead {
                    address,
                    size: 2,
                    cause: e,
                }),
            }
        } else {
            Err(RunError::MemRead {
                address,
                size: 2,
                cause: MemoryAccessError::InvalidAddress,
            })
        }
    }

    /// Reads 32-bit word at `address` without checking for privileges or alignment.
    pub fn read_u32le_iface(&mut self, address: u32) -> Result<u32, RunError> {
        let mut env = Env::new(self.cycles, self.is_privileged());
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemRead {
                address,
                size: 4,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let read = mapping
            .iface
            .borrow_mut()
            .read_u32le(address - mapping.address, &mut env);
        self.memory_op_actions.extend(env.actions);
        match read {
            Ok(val) => Ok(val),
            Err(e) => Err(RunError::MemRead {
                address,
                size: 4,
                cause: e,
            }),
        }
    }

    /// Writes 32-bit word at `address` without checking for privileges or alignment.
    pub fn write_u32le_iface(&mut self, address: u32, value: u32) -> Result<(), RunError> {
        let mut env = Env::new(self.cycles, self.is_privileged());
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemWrite {
                address,
                size: 4,
                value,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let write =
            mapping
                .iface
                .borrow_mut()
                .write_u32le(address - mapping.address, value, &mut env);
        self.memory_op_actions.extend(env.actions);
        match write {
            Ok(()) => Ok(()),
            Err(e) => Err(RunError::MemWrite {
                address,
                size: 4,
                value,
                cause: e,
            }),
        }
    }

    /// Corresponds to `ValidateAddress()` in the Arm Architecture Reference Manual.
    /// Currently not implemented, MPU is not enforced yet.
    fn validate_address(
        &self,
        _address: u32,
        _is_priv: bool,
        _is_write: bool,
        _is_instr_fetch: bool,
    ) {
        // TODO
    }

    /// Reads `size` successive bytes starting at `address`, without checking for privileges or
    /// alignment.
    ///
    /// Panics if address overflows. This may change in the future.
    pub fn read_bytes_iface(&mut self, address: u32, size: u32) -> Result<Vec<u8>, RunError> {
        let mut result = Vec::new();
        result.reserve_exact(size as usize);
        for a in address..address.checked_add(size).unwrap() {
            result.push(self.read_u8_iface(a)?);
        }
        Ok(result)
    }

    /// Writes bytes starting at `address`, without checking for privileges or alignment.
    ///
    /// Panics if address overflows. This may change in the future.
    pub fn write_bytes_iface(&mut self, address: u32, data: &[u8]) -> Result<(), RunError> {
        for i in 0..data.len() as u32 {
            self.write_u8_iface(address.checked_add(i).unwrap(), data[i as usize])?
        }
        Ok(())
    }

    /// Returns current value of the Stack Pointer (r13)
    pub fn sp(&self) -> u32 {
        self.registers[13]
    }

    /// Sets the value of a register.
    ///
    /// This is a shorcut to `self.registers.set(r, value)`.
    pub fn set(&mut self, index: RegisterIndex, value: u32) {
        self.registers.set(index, value)
    }

    /// Sets value of the Stack Pointer (r13)
    ///
    /// # Arguments
    ///
    /// * `value` - New Stack Pointer value
    pub fn set_sp(&mut self, value: u32) {
        *self.registers.sp_mut() = value
    }

    /// Returns current value of the Link Register (r14)
    pub fn lr(&self) -> u32 {
        self.registers[14]
    }

    /// Sets Link Register (r14) value
    ///
    /// # Arguments
    ///
    /// * `value` - New Link Register value
    pub fn set_lr(&mut self, value: u32) {
        self.registers.lr = value
    }

    /// Returns current value of the Program Counter (r15)
    pub fn pc(&self) -> u32 {
        self.registers.pc
    }

    /// Sets Program Counter (r15) value. This is equivalent to modifying [CoreRegisters::pc] in
    /// [Self::registers].
    pub fn set_pc(&mut self, value: u32) {
        self.registers.pc = value;
    }

    fn decode_instruction(
        &mut self,
        address: u32,
    ) -> Result<(InstructionBox, InstructionSize), RunError> {
        let hw = self.read_u16le_iface(address)?;
        let it_state = self.registers.psr.it_state();
        let size = InstructionSize::from_halfword(hw);
        let ins = match size {
            InstructionSize::Ins16 => {
                self.instruction_decoder
                    .try_decode(hw as u32, InstructionSize::Ins16, it_state)?
            }
            InstructionSize::Ins32 => {
                let hw2 = self.read_u16le_iface(address + 2)?;
                self.instruction_decoder.try_decode(
                    ((hw as u32) << 16) + hw2 as u32,
                    InstructionSize::Ins32,
                    it_state,
                )?
            }
        };
        Ok((ins, size))
    }

    fn step(&mut self) -> Result<(), RunError> {
        // Handle interrupt requests
        if let Some(irq) = self.interrupt_requests.pop_first() {
            let max_num = self.exception_active.len();
            let num = irq.number();
            assert!(
                (num as usize) < max_num,
                "Exception number too high: got {}, max is {}",
                num,
                max_num - 1
            );
            // TODO: not all exceptions may result in a WFI wakeup event.
            if self.state == State::WaitingForInterrupt {
                self.state = State::Running;
            }
            if !self.exception_active[irq.number() as usize] {
                self.exception_entry(irq)?;
            }
        }

        // Handle hooks
        let pc = self.pc();
        if self
            .code_hooks
            .iter()
            .any(|ch| ch.range.contains(&(pc as usize)))
        {
            self.events.push(Event::Hook { address: pc });
            return Ok(());
        }

        match self.state {
            State::Running => {
                let (ins, effect) = self.execute_next_instruction()?;
                self.events.push(Event::Instruction { ins });
                match effect {
                    Effect::None => {}
                    Effect::Branch => {}
                    Effect::Break(i) => self.events.push(Event::Break(i)),
                    Effect::DebugHint(i) => self.events.push(Event::DebugHint(i)),
                    Effect::WaitForEvent => self.state = State::WaitingForEvent,
                    Effect::WaitForInterrupt => self.state = State::WaitingForInterrupt,
                }
            }
            State::WaitingForEvent => {
                if self.registers.event {
                    // Leave wait state to resume execution.
                    // Arm Architecture Reference Manual says clearing the event register is
                    // implementation defined. We do clear it here, I don't see how it could work
                    // otherwise...
                    self.registers.event = false;
                    self.state = State::Running;
                }
            }
            State::WaitingForInterrupt => {}
        }

        // Handle actions that may come from memory accesses.
        for action in self.memory_op_actions.iter() {
            match action {
                MemoryOpAction::Reset => self.events.push(Event::Reset),
                MemoryOpAction::Irq(irq) => {
                    // A peripheral emitted an interrupt request, save it.
                    self.interrupt_requests.insert(*irq);
                }
                MemoryOpAction::Update(_) => panic!(), // This should be filtered prior
            }
        }
        self.memory_op_actions.clear();
        self.update_peripherals();
        self.cycles += 1;
        Ok(())
    }

    fn execute_next_instruction(&mut self) -> Result<(InstructionBox, Effect), RunError> {
        let (ins, size) = self.decode_instruction(self.pc())?;
        // PC is always 4 bytes ahead of currently executed instruction, so we increment PC before
        // applying the effect of the instruction, and we go back 2 bytes if this is a 16-bit
        // instruction.
        self.set_pc(self.pc() + 4);
        // Conditional execution testing.
        // Condition is usually defined by the current IT state, excepted for conditional
        // branch B instruction which can override it by implementing [Instruction::condition].
        let mut it_state = self.registers.psr.it_state();
        let condition = ins
            .condition()
            .or(it_state.current_condition())
            .unwrap_or(Condition::Always);
        // Advance IT block state. This must be done before executing the instruction, because if
        // the current instruction is IT, it will load a new state.
        it_state.advance();
        self.registers.psr.set_it_state(it_state);

        let effect = if self.registers.psr.test(condition) {
            ins.execute(self)?
        } else {
            Effect::None
        };

        if effect != Effect::Branch && size == InstructionSize::Ins16 {
            self.set_pc(self.pc() - 2)
        }

        Ok((ins, effect))
    }

    /// Call `update` on memory mapping which requested an update during a previous operation.
    pub fn update_peripherals(&mut self) {
        let mut env = Env::new(self.cycles, self.is_privileged());
        for mapping in self.memory_mappings.0.iter_mut() {
            mapping.iface.borrow_mut().update(&mut env);
        }
        self.memory_op_actions.extend(env.actions);
    }

    pub fn request_interrupt(&mut self, irq: Irq) {
        self.interrupt_requests.insert(irq);
    }

    /// Enters exception.
    ///
    /// Corresponds to the function `ExceptionEntry()` described in the ARM Architecture Reference
    /// Manual.
    fn exception_entry(&mut self, number: Irq) -> Result<(), RunError> {
        self.push_stack()?;
        self.exception_taken(number)?;
        Ok(())
    }

    /// Returns from exception.
    ///
    /// Corresponds to the function `ExceptionReturn()` described in the ARM Architecture Reference
    /// Manual.
    fn exception_return(&mut self, exc_return: u32) -> Result<(), RunError> {
        assert_eq!(self.registers.mode, Mode::Handler);
        ////unpredictable(address & 0xf0000000 != 0xf0000000)?;
        let number = self.registers.psr.exception_number();
        let nested_activation = false; // TODO
        if !self.exception_active[number as usize] {
            self.deactivate(number);
            todo!();
            //return Ok(());
        }
        let frame_ptr = match exc_return & 0xf {
            0b0001 => {
                // Return to Handler
                self.registers.mode = Mode::Handler;
                self.registers.control.set_spsel(false);
                self.registers.msp
            }
            0b1001 => {
                // Returning to Thread using Main stack
                if nested_activation && !self.system_control.borrow().ccr.nonbasethrdena() {
                    todo!()
                } else {
                    self.registers.mode = Mode::Thread;
                    self.registers.control.set_spsel(false);
                    self.registers.msp
                }
            }
            0b1101 => {
                // Returning to Thread using Process stack
                if nested_activation && !self.system_control.borrow().ccr.nonbasethrdena() {
                    todo!()
                } else {
                    self.registers.mode = Mode::Thread;
                    self.registers.control.set_spsel(true);
                    self.registers.psp
                }
            }
            _ => {
                self.deactivate(number);
                todo!()
            }
        };
        self.deactivate(number);
        self.pop_stack(frame_ptr, exc_return)?;

        if self.registers.mode == Mode::Handler && self.registers.psr.ipsr() == 0 {
            todo!();
        }

        if self.registers.mode == Mode::Thread && self.registers.psr.ipsr() != 0 {
            todo!();
        }

        // TODO ClearExclusiveLocal()
        self.registers.event = true;
        Ok(())
    }

    /// Deactivates an exception
    fn deactivate(&mut self, number: u16) {
        self.exception_active[number as usize] = false;
        if self.registers.psr.exception_number() != 2 {
            // 2 is NMI
            self.registers.faultmask.set_pm(false);
        }
    }

    /// On exception entry, save some registers on stack.
    ///
    /// Corresponds to `PushStack()` in ARM architecture reference manual.
    fn push_stack(&mut self) -> Result<(), RunError> {
        let frame_size = 0x20;
        let force_align = self.system_control.borrow().ccr.stkalign();
        let sp_mask = !((force_align as u32) << 2);
        let frame_ptr_align = self.sp().bit(2) && force_align;
        let frame_ptr = (self.sp() - frame_size) & sp_mask;
        self.set_sp(frame_ptr);

        let return_address = self.pc();
        self.write_u32le_iface(frame_ptr, self.registers.r0)?;
        self.write_u32le_iface(frame_ptr + 0x04, self.registers.r1)?;
        self.write_u32le_iface(frame_ptr + 0x08, self.registers.r2)?;
        self.write_u32le_iface(frame_ptr + 0x0c, self.registers.r3)?;
        self.write_u32le_iface(frame_ptr + 0x10, self.registers.r12)?;
        self.write_u32le_iface(frame_ptr + 0x14, self.registers.lr)?;
        self.write_u32le_iface(frame_ptr + 0x18, return_address)?;
        let mut xpsr = self.registers.psr.get();
        xpsr.set_bit(9, frame_ptr_align);
        self.write_u32le_iface(frame_ptr + 0x1c, xpsr)?;

        let lr = match self.registers.mode {
            Mode::Handler => 0xfffffff1,
            Mode::Thread => {
                if !self.registers.control.spsel() {
                    0xfffffff9
                } else {
                    0xfffffffd
                }
            }
        };
        self.set_lr(lr);

        Ok(())
    }

    /// On exception return, restore some registers from the stack.
    ///
    /// This corresponds to `PopStack()` from ARM architecture reference manual.
    fn pop_stack(&mut self, frame_ptr: u32, exc_return: u32) -> Result<(), RunError> {
        let frame_size = 0x20;
        let force_align = self.system_control.borrow().ccr.stkalign();
        self.registers.r0 = self.read_u32le_iface(frame_ptr)?;
        self.registers.r1 = self.read_u32le_iface(frame_ptr + 0x04)?;
        self.registers.r2 = self.read_u32le_iface(frame_ptr + 0x08)?;
        self.registers.r3 = self.read_u32le_iface(frame_ptr + 0x0c)?;
        self.registers.r12 = self.read_u32le_iface(frame_ptr + 0x10)?;
        self.registers.lr = self.read_u32le_iface(frame_ptr + 0x14)?;
        let mut pc = self.read_u32le_iface(frame_ptr + 0x18)?;
        // PC should be halfword aligned, otherwise execution is unpredictable according to the
        // specification. However, some implementations may not respect this and it may still work
        // on hardware, so we have the option `tolerate_pop_stack_unaligned_pc` to tolerate this if
        // needed.
        if pc % 2 != 0 {
            if !self.tolerate_pop_stack_unaligned_pc {
                return Err(RunError::Unpredictable);
            }
            pc &= 0xfffffffe;
        };
        self.registers.pc = pc;

        let psr = self.read_u32le_iface(frame_ptr + 0x1c)?;
        let sp_mask = ((psr.bit(9) && force_align) as u32) << 2;
        self.registers.psr.set(psr); // Note: this does not copy bit 9

        match exc_return & 0xf {
            0b0001 | 0b1001 | 0b1101 => {
                let new_sp = (self.sp() + frame_size) | sp_mask;
                self.set_sp(new_sp);
            }
            _ => {}
        }

        self.registers.psr.set(psr & 0xfff0ffff); // TODO remove mask if FP extension
        Ok(())
    }

    fn exception_taken(&mut self, number: Irq) -> Result<(), RunError> {
        let vtor = self.system_control.borrow().vtor.offset();
        let vector_address = number.number() as u32 * 4 + vtor;
        let jump_address = self.read_u32le_iface(vector_address)?;
        self.set_pc(jump_address & 0xfffffffe);
        self.registers.mode = Mode::Handler;
        self.registers
            .psr
            .set_exception_number(number.number())
            .set_t(jump_address & 1 != 0)
            .set_ici_it(0);
        // TODO: Set CONTROL.FPCA to 1 if FP available
        self.registers.control.set_spsel(false);
        self.exception_active[number.number() as usize] = true;
        // TODO: SCS_UpdateStatusRegs()
        // TODO: ClearExclusiveLocal()
        self.registers.event = true; // SetEventRegister()

        // TODO: InstructionSynchronizationBarrier()
        Ok(())
    }

    /// Returns true if execution is privileged.
    ///
    /// Corresponds to `FindPriv()` from Arm Architecture Reference Manual.
    pub fn is_privileged(&self) -> bool {
        if self.registers.mode == Mode::Handler {
            true
        } else {
            !self.registers.control.privileged_bit()
        }
    }

    /// Write value to PC, with interworking for ARM only from ARMv7
    pub fn alu_write_pc(&mut self, address: u32) {
        self.blx_write_pc(address)
    }

    /// Write value to PC, with interworking
    pub fn blx_write_pc(&mut self, address: u32) {
        self.registers.psr.set_t(address & 1 == 1);
        self.set_pc(address & 0xfffffffe)
    }

    /// Corresponds to operation `LoadWritePC()` described in ARMv7-M Architecture Reference Manual.
    pub fn load_write_pc(&mut self, address: u32) -> Result<(), RunError> {
        self.bx_write_pc(address)
    }

    /// Write value to PC, with interworking
    pub fn bx_write_pc(&mut self, address: u32) -> Result<(), RunError> {
        if self.registers.mode == Mode::Handler && (address >> 28 == 0xf) {
            self.exception_return(address & 0x0fffffff)
        } else {
            self.blx_write_pc(address);
            Ok(())
        }
    }

    pub fn condition_passed(&self) -> bool {
        if let Some(condition) = self.registers.psr.it_state().current_condition() {
            self.registers.psr.test(condition)
        } else {
            true
        }
    }

    /// Calls for [Coprocessor::accepted] of a selected coprocessor, for a given instruction.
    ///
    /// If coprocessor does not accept instruction, or if coprocessor is not defined,
    /// [None] is returned.
    ///
    /// If coprocessor is defined and accepts the instruction, it is returned.
    ///
    /// This corresponds to `Coproc_Accepted()` ins Arm Architecture Reference Manual.
    pub fn coproc_accepted(&mut self, cp: u8, ins: u32) -> Option<Rc<RefCell<dyn Coprocessor>>> {
        debug_assert!(cp < 16);
        let coprocessor = self.coprocessors[cp as usize].clone()?;
        if !coprocessor.borrow().accepted(ins) {
            return None;
        }
        Some(coprocessor)
    }

    /// Raise usage fault exception and sets NOCP fault flag.
    pub fn generate_coprocessor_exception(&mut self) {
        self.request_interrupt(Irq::UsageFault);
        self.system_control.borrow_mut().cfsr.set_nocp(true);
    }

    /// Tags a memory address for exclusive access. `size` can be in [1, 4].
    ///
    /// Corresponds to `SetExclusiveMonitors()` in Arm Architecture Reference Manual.
    pub fn set_exclusive_monitors(&mut self, address: u32, size: u32) {
        debug_assert!((size >= 1) && (size <= 4));
        let granule = self.local_monitor.granule;
        self.local_monitor.state = MonitorState::ExclusiveAccess {
            address: address.align(granule as usize),
        };
    }

    /// If the local exclusive monitor has locked a given address, clear the lock and return true.
    ///
    /// Corresponds to `ExclusiveMonitorsPass()` in the Arm Architecture Reference Manual.
    pub fn exclusive_monitors_pass(&mut self, address: u32, size: u32) -> Result<bool, RunError> {
        self.usage_fault_if_unaligned(address, size as usize)?;
        // TODO address validation
        if self.local_monitor.state == (MonitorState::ExclusiveAccess { address: address }) {
            self.clear_exclusive_local();
            return Ok(true);
        }
        return Ok(false);
    }

    /// Clears the local exclusive monitor lock.
    ///
    /// Corresponds to `ClearExclusiveLocal()` in the Arm Architecture Reference Manual.
    pub fn clear_exclusive_local(&mut self) {
        self.local_monitor.state = MonitorState::OpenAccess;
    }
}

/// Options passed to [Emulator::run] to configure emulation.
#[derive(Default)]
pub struct RunOptions {
    /// Limit of the number of instructions to be executed.
    gas: Option<usize>,
}

impl RunOptions {
    pub fn new() -> Self {
        Self { gas: None }
    }

    /// Sets a limit to the number of instructions to be executed.
    pub fn gas(mut self, gas: usize) -> Self {
        self.gas = Some(gas);
        self
    }
}

/// [Processor] implements this trait to provide an interface for running the device with
/// possible options.
///
/// The most important method is [Emulator::next_event] which runs a single emulation step and
/// returns the first event encountered. A device struct using composition with [Processor] may
/// implement this trait and the [Emulator::next_event] method to handle, filter or forward
/// processor events and this way build an additive layer to the processor (hooks are a good
/// example use case). This way the new struct will automatically benefit from [Emulator::run]
/// blanket implementation and provide the same interface as [Processor].
pub trait Emulator {
    /// Processes execution until next event, and returns that event.
    fn next_event(&mut self) -> Result<Event, RunError>;

    /// Runs the device until a non instruction event occurs of emulation reaches optional gas
    /// limit.
    fn run(&mut self, options: RunOptions) -> Result<Option<Event>, RunError> {
        let mut ins_count = 0;
        loop {
            if let Some(count) = options.gas {
                if ins_count == count {
                    return Ok(None);
                }
            }
            let event = self.next_event()?;
            match event {
                Event::Hook { address: _ }
                | Event::Reset
                | Event::Break(_)
                | Event::DebugHint(_) => return Ok(Some(event.clone())),
                Event::Instruction { ins: _ } => ins_count += 1,
            }
        }
    }
}

impl Emulator for Processor {
    fn next_event(&mut self) -> Result<Event, RunError> {
        while self.events.is_empty() {
            self.step()?;
        }
        Ok(self.events.pop().unwrap())
    }
}

/// Indexing implemented for easier access to the registers.
impl Index<RegisterIndex> for Processor {
    type Output = u32;

    fn index(&self, index: RegisterIndex) -> &Self::Output {
        &self.registers[index]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Running,
    WaitingForEvent,
    WaitingForInterrupt,
}

/// An instruction execution may result in some optional effect that require special treatment
/// during execution. This enum is used to report those.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Instruction has no particular effect. Most instructions will result in this enumeration
    /// value when executed.
    None,
    /// Instruction has modified the PC register.
    /// When this enumeration value is returned, the execution will not increment the PC after
    /// executing the instruction.
    Branch,
    /// Returned by the BKPT instruction.
    Break(u8),
    /// Returned by DBG instruction.
    DebugHint(u8),
    /// Instruction requests the halt of the processor execution until an event occurs.
    WaitForEvent,
    /// Instruction requests the halt of the processor execution until an interrupt occurs.
    WaitForInterrupt,
}

/// Error returned when mapping an interface would intersect with an already mapping other
/// interface.
#[derive(Debug)]
pub struct MapConflict;
