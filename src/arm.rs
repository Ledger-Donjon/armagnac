use core::panic;
use std::{
    cell::RefCell,
    collections::BTreeSet,
    ops::{Index, Range},
    rc::Rc,
};

use crate::{
    arith::add_with_carry,
    condition::Condition,
    decoder::{ArmV7InstructionDecoder, InstructionDecodeError},
    helpers::BitAccess,
    instructions::{thumb_ins_size, unpredictable, Instruction, InstructionSize},
    irq::Irq,
    it_state::ItState,
    memory::{Env, MemoryAccessError, MemoryInterface, MemoryOpAction, RamMemory},
    memory_protection_unit::MemoryProtectionUnitV8M,
    registers::{CoreRegisters, Mode, RegisterIndex},
    system_control::SystemControl,
};

struct MemoryMap {
    address: u32,
    size: u32,
    iface: Rc<RefCell<dyn MemoryInterface>>,
}

#[derive(Debug, Copy, Clone)]
pub enum RunError {
    InstructionUnknown,
    InstructionUnpredictable,
    /// Execution leads to unpredictable result.
    Unpredictable,
    MemRead {
        address: u32,
        size: u32,
        cause: MemoryAccessError,
    },
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

pub enum Event {
    Hook {
        address: u32,
    },
    Instruction {
        ins: InstructionBox,
    },
    /// Software reset issued from an instruction emulation
    ResetFromInstruction {
        ins: InstructionBox,
    },
    /// Reset requested during a peripheral emulation, before next instruction could be executed
    ResetFromPeripheral,
    Break,
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
                return Some(&mapping);
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

#[derive(Clone, Copy)]
pub enum ArmVersion {
    V7,
    V8M,
}

/// ARMv7-M processor state.
pub struct Arm7Processor {
    /// ARM emutaled version.
    pub version: ArmVersion,
    /// r0-r15 and sys registers.
    pub registers: CoreRegisters,
    pub execution_priority: i16,
    /// Indicates which exceptions are currently active.
    exception_active: Vec<bool>,
    memory_mappings: MemoryMappings,
    instruction_decoder: ArmV7InstructionDecoder,
    pub cycles: u64,
    code_hooks: Vec<CodeHook>,
    pub event_on_instruction: bool,
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
    /// When poping PC from the stack (during exception return for instance), PC should be aligned,
    /// otherwise execution is unpredictable according to the ARM specification. However, some
    /// implementation may still set LSB to 1 for Thumb mode, and this can work on some hardware.
    /// So to allow emulation in that case, `tolerate_pop_stack_unaligned_pc` can be set to `true`.
    /// If `false` (the default) an error will be reported by the emulation if PC is unaligned.
    pub tolerate_pop_stack_unaligned_pc: bool,
}

type InstructionBox = Box<dyn Instruction>;

pub trait Mnemonic {
    fn mnemonic(&self, pc: u32) -> String;
}

impl Mnemonic for Box<dyn Instruction> {
    fn mnemonic(&self, pc: u32) -> String {
        format!("{:<6} {}", self.name(), self.args(pc))
    }
}

impl Arm7Processor {
    /// Creates a new ARMv7 processor
    ///
    /// # Arguments
    ///
    /// * `external_exception_count` - Number of available external exceptions.
    pub fn new(version: ArmVersion, external_exception_count: usize) -> Arm7Processor {
        let exception_count = 16usize.checked_add(external_exception_count).unwrap();
        let system_control = Rc::new(RefCell::new(SystemControl::new()));
        let mut processor = Arm7Processor {
            version,
            registers: CoreRegisters::new(),
            memory_mappings: MemoryMappings::new(),
            execution_priority: 0,
            exception_active: (0..exception_count).map(|_| false).collect(),
            instruction_decoder: ArmV7InstructionDecoder::new(),
            cycles: 0,
            code_hooks: Vec::new(),
            event_on_instruction: false,
            memory_op_actions: Vec::new(),
            interrupt_requests: BTreeSet::new(),
            system_control: system_control.clone(),
            tolerate_pop_stack_unaligned_pc: false,
        };
        processor.map_iface(0xe000e000, system_control).unwrap();
        match version {
            ArmVersion::V7 => {}
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

    pub fn map(&mut self, address: u32, data: &[u8]) -> Result<(), ()> {
        self.map_iface(
            address,
            Rc::new(RefCell::new(RamMemory::new_from_slice(data))),
        )
    }

    /// Maps an interface to the memory space. Returns an error if the new interface size overflows
    /// the address space, or if the new interface memory region intersects with an already mapped
    /// memory region.
    pub fn map_iface(
        &mut self,
        address: u32,
        iface: Rc<RefCell<dyn MemoryInterface>>,
    ) -> Result<(), ()> {
        let size = iface.borrow().size();

        // Check for size overflow
        if address.checked_add(size).is_none() {
            return Err(());
        }

        // Check overlap with another mapping
        if self
            .memory_mappings
            .0
            .iter()
            .find(|m| {
                let a = m.address.max(address);
                let b = (m.address + m.size).min(address + size);
                a < b
            })
            .is_some()
        {
            return Err(());
        }

        self.memory_mappings.0.push(MemoryMap {
            address: address,
            size,
            iface: iface,
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
    pub fn map_ram(&mut self, address: u32, size: u32) -> Result<Rc<RefCell<RamMemory>>, ()> {
        let ram = Rc::new(RefCell::new(RamMemory::new_zero(size as usize)));
        self.map_iface(address, ram.clone())?;
        Ok(ram)
    }

    pub fn hook_code(&mut self, range: Range<usize>) {
        self.code_hooks.push(CodeHook { range: range })
    }

    /// Returns bytes at given address in memory
    pub fn u8_at(&mut self, address: u32) -> Result<u8, RunError> {
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
                address: address,
                size: 1,
                cause: e,
            }),
        }
    }

    ///// Set byte value at address
    pub fn set_u8_at(&mut self, address: u32, value: u8) -> Result<(), RunError> {
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
                address: address,
                size: 1,
                value: value as u32,
                cause: e,
            }),
        }
    }

    pub fn set_u16le_at(&mut self, address: u32, value: u16) -> Result<(), RunError> {
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
                address: address,
                size: 2,
                value: value as u32,
                cause: e,
            }),
        }
    }

    /// Returns halfword at given address in memory
    pub fn u16le_at(&mut self, address: u32) -> Result<u16, RunError> {
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
                    address: address,
                    size: 2,
                    cause: e,
                }),
            }
        } else {
            Err(RunError::MemRead {
                address: address,
                size: 2,
                cause: MemoryAccessError::InvalidAddress,
            })
        }
    }

    /// Returns 32-bit word at given address in memory
    pub fn u32le_at(&mut self, address: u32) -> Result<u32, RunError> {
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
                address: address,
                size: 4,
                cause: e,
            }),
        }
    }

    pub fn set_u32le_at(&mut self, address: u32, value: u32) -> Result<(), RunError> {
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
        write.map_err(|e| RunError::MemWrite {
            address,
            size: 4,
            value,
            cause: e,
        })
    }

    /// Reads `size` successive bytes starting at `address`.
    ///
    /// Panics if address overflows. This may change in the future.
    pub fn bytes_at(&mut self, address: u32, size: u32) -> Result<Vec<u8>, RunError> {
        let mut result = Vec::new();
        result.reserve_exact(size as usize);
        for a in address..address.checked_add(size).unwrap() {
            result.push(self.u8_at(a)?);
        }
        Ok(result)
    }

    /// Writes bytes starting at `address`.
    ///
    /// Panics if address overflows. This may change in the future.
    pub fn set_bytes_at(&mut self, address: u32, data: &[u8]) -> Result<(), RunError> {
        for i in 0..data.len() as u32 {
            self.set_u8_at(address.checked_add(i).unwrap(), data[i as usize])?
        }
        Ok(())
    }

    /// Returns current value of the Stack Pointer (r13)
    pub fn sp(&self) -> u32 {
        self.registers[13]
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

    /// Sets Program Counter (r15) value
    ///
    /// # Arguments
    ///
    /// * `value` - New Program Counter value
    pub fn set_pc(&mut self, value: u32) {
        self.registers.pc = value;
    }

    fn decode_instruction(
        &mut self,
        address: u32,
    ) -> Result<(InstructionBox, InstructionSize), RunError> {
        let hw = self.u16le_at(address)?;
        let it_state = self.registers.xpsr.it_state();
        let (ins, size) = match thumb_ins_size(hw) {
            InstructionSize::Ins16 => (
                self.instruction_decoder
                    .try_decode(hw as u32, InstructionSize::Ins16, it_state)?,
                InstructionSize::Ins16,
            ),
            InstructionSize::Ins32 => {
                let hw2 = self.u16le_at(address + 2)?;
                (
                    self.instruction_decoder.try_decode(
                        ((hw as u32) << 16) + hw2 as u32,
                        InstructionSize::Ins32,
                        it_state,
                    )?,
                    InstructionSize::Ins32,
                )
            }
        };
        Ok((ins, size))
    }

    /// Call `update` on memory mapping which requested an update during a previous operation.
    pub fn update_peripherals(&mut self) -> Option<Event> {
        let mut env = Env::new(self.cycles, self.is_privileged());
        for mapping in self.memory_mappings.0.iter_mut() {
            mapping.iface.borrow_mut().update(&mut env);
        }
        self.memory_op_actions.extend(env.actions);
        None
    }

    pub fn request_interrupt(&mut self, irq: Irq) {
        self.interrupt_requests.insert(irq);
    }

    pub fn stepi(&mut self) -> Result<Event, RunError> {
        if let Some(event) = self.update_peripherals() {
            return Ok(event);
        }

        let pc = self.pc();

        if self
            .code_hooks
            .iter()
            .find(|&ch| ch.range.contains(&(pc as usize)))
            .is_some()
        {
            Ok(Event::Hook { address: pc })
        } else {
            let (ins, size) = self.decode_instruction(self.pc())?;
            // PC is always 4 bytes ahead of currently executed instruction, so we increment PC before
            // applying the effect of the instruction, and we go back 2 bytes if this is a 16-bit
            // instruction.
            self.set_pc(self.pc() + 4);
            // Conditional execution testing.
            // Condition is usually defined by the current IT state, excepted for conditional
            // branch B instruction which can override it by implementing [Instruction::condition].
            let mut it_state = self.registers.xpsr.it_state();
            let condition = ins
                .condition()
                .or(it_state.current_condition())
                .unwrap_or(Condition::Always);
            // Advance IT block state. This must be done before executing the instruction,
            // because if the current instruction is IT, it will load a new state.
            it_state.advance();
            self.registers.xpsr.set_it_state(it_state);

            let mut jump = false;
            if self.registers.xpsr.test(condition) {
                jump = ins.execute(self)?;
            }
            if !jump {
                if size == InstructionSize::Ins16 {
                    self.set_pc(self.pc() - 2)
                }
            }
            // Handle actions that may come from memory accesses.
            for action in self.memory_op_actions.iter() {
                match action {
                    MemoryOpAction::Reset => return Ok(Event::ResetFromInstruction { ins: ins }),
                    MemoryOpAction::Irq(irq) => {
                        // A peripheral emitted an interrupt request, save it.
                        self.interrupt_requests.insert(*irq);
                    }
                    MemoryOpAction::Update(_) => panic!(), // This should be filtered prior
                }
            }
            self.memory_op_actions.clear();
            self.cycles += 1;

            // TODO: handle priorities
            if let Some(irq) = self.interrupt_requests.pop_first() {
                let max_num = self.exception_active.len();
                let num = irq.number();
                assert!(
                    (num as usize) < max_num,
                    "Exception number too high: got {}, max is {}",
                    num,
                    max_num - 1
                );
                if !self.exception_active[irq.number() as usize] {
                    self.exception_entry(irq)?;
                }
            }

            Ok(Event::Instruction { ins })
        }
    }

    fn exception_entry(&mut self, number: Irq) -> Result<(), RunError> {
        self.push_stack()?;
        self.exception_taken(number);
        Ok(())
    }

    /// Returns from exception.
    ///
    /// Corresponds to the function `ExceptionReturn()` described in the ARM Architecture Reference
    /// Manual.
    fn exception_return(&mut self, exc_return: u32) -> Result<(), RunError> {
        assert!(self.registers.mode == Mode::Handler);
        ////unpredictable(address & 0xf0000000 != 0xf0000000)?;
        let number = self.registers.xpsr.exception_number();
        let nested_activation = false; // TODO
        if !self.exception_active[number as usize] {
            self.deactivate(number);
            todo!();
            return Ok(());
        }
        let frame_ptr = match exc_return & 0xf {
            0b0001 => todo!(),
            0b1001 => todo!(),
            0b1101 => {
                // Returning to thread using process stack
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

        if self.registers.mode == Mode::Handler && self.registers.xpsr.ipsr() == 0 {
            todo!();
        }

        if self.registers.mode == Mode::Thread && self.registers.xpsr.ipsr() != 0 {
            todo!();
        }

        // TODO ClearExclusiveLocal()
        // TODO SetEventRegister()
        Ok(())
    }

    /// Deactivates an exception
    fn deactivate(&mut self, number: u16) {
        self.exception_active[number as usize] = false;
        if self.registers.xpsr.exception_number() != 2 {
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
        println!("DEBUG SET RETURN ADDRESS {:0x}", return_address);
        self.set_u32le_at(frame_ptr, self.registers.r0)?;
        self.set_u32le_at(frame_ptr + 0x04, self.registers.r1)?;
        self.set_u32le_at(frame_ptr + 0x08, self.registers.r2)?;
        self.set_u32le_at(frame_ptr + 0x0c, self.registers.r3)?;
        self.set_u32le_at(frame_ptr + 0x10, self.registers.r12)?;
        self.set_u32le_at(frame_ptr + 0x14, self.registers.lr)?;
        self.set_u32le_at(frame_ptr + 0x18, return_address)?;
        let mut xpsr = self.registers.xpsr.get();
        xpsr.set_bit(9, frame_ptr_align);
        self.set_u32le_at(frame_ptr + 0x1c, xpsr)?;

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
        self.registers.r0 = self.u32le_at(frame_ptr)?;
        self.registers.r1 = self.u32le_at(frame_ptr + 0x04)?;
        self.registers.r2 = self.u32le_at(frame_ptr + 0x08)?;
        self.registers.r3 = self.u32le_at(frame_ptr + 0x0c)?;
        self.registers.r12 = self.u32le_at(frame_ptr + 0x10)?;
        self.registers.lr = self.u32le_at(frame_ptr + 0x14)?;
        let mut pc = self.u32le_at(frame_ptr + 0x18)?;
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
        let psr = self.u32le_at(frame_ptr + 0x1c)?;

        let sp_mask = ((self.registers.xpsr.get().bit(9) && force_align) as u32) << 2;
        match exc_return & 0xf {
            0b0001 | 0b1001 | 0b1101 => {
                let new_sp = (self.sp() + frame_size) | sp_mask;
                self.set_sp(new_sp);
            }
            _ => {}
        }

        self.registers.xpsr.set(psr & 0xfff0ffff); // TODO remove mask if FP extension
        Ok(())
    }

    fn exception_taken(&mut self, number: Irq) {
        let vtor = self.system_control.borrow().vtor.offset();
        let vector_address = number.number() as u32 * 4 + vtor;
        let jump_address = self.u32le_at(vector_address).unwrap();
        self.set_pc(jump_address & 0xfffffffe);
        self.registers.mode = Mode::Handler;
        self.registers
            .xpsr
            .set_exception_number(number.number())
            .set_t(jump_address & 1 != 0)
            .set_ici_it(0);
        // TODO: Set CONTROL.FPCA to 1 if FP available
        self.registers.control.set_spsel(false);
        self.exception_active[number.number() as usize] = true;
        // TODO: SCS_UpdateStatusRegs()
        // TODO: ClearExclusiveLocal()
        // TODO: SetEventRegister()
        // TODO: InstructionSynchronizationBarrier()
    }

    /// Run until next event and return the event
    pub fn run(&mut self) -> Result<Event, RunError> {
        loop {
            let e = self.stepi()?;
            match e {
                Event::Instruction { ins } => {
                    if self.event_on_instruction {
                        return Ok(Event::Instruction { ins });
                    }
                }
                _ => return Ok(e),
            }
        }
    }

    /// Returns true if execution is privileged
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
        self.registers.xpsr.set_t(address & 1 == 1);
        self.set_pc(address & 0xfffffffe)
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
        if let Some(condition) = self.registers.xpsr.it_state().current_condition() {
            self.registers.xpsr.test(condition)
        } else {
            true
        }
    }
}

/// Indexing implemented for easier access to the registers.
impl Index<RegisterIndex> for Arm7Processor {
    type Output = u32;

    fn index(&self, index: RegisterIndex) -> &Self::Output {
        &self.registers[index]
    }
}
