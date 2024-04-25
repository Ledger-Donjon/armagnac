use std::{
    cell::RefCell,
    ops::{Index, IndexMut, Range},
    rc::Rc,
};

use crate::{
    arith::add_with_carry,
    condition::Condition,
    decoder::{ArmV7InstructionDecoder, InstructionDecodeError},
    instructions::{thumb_ins_size, Instruction, InstructionSize},
    it_state::ItState,
    memory::{Env, MemoryAccessError, MemoryInterface, MemoryOpAction, RamMemory},
    registers::{CoreRegisters, Register, RegisterIndex},
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
    MemRead {
        address: u32,
        size: u32,
        cause: MemoryAccessError,
    },
    MemWrite {
        address: u32,
        size: u32,
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

/// ARMv7-M processor state.
pub struct Arm7Processor {
    /// r0-r15 registers.
    pub registers: CoreRegisters,
    pub execution_priority: i16,
    /// Current IT block state.
    pub it_state: ItState,
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
    pub fn new() -> Arm7Processor {
        let mut processor = Arm7Processor {
            registers: CoreRegisters::new(),
            memory_mappings: MemoryMappings::new(),
            execution_priority: 0,
            it_state: ItState::new(),
            instruction_decoder: ArmV7InstructionDecoder::new(),
            cycles: 0,
            code_hooks: Vec::new(),
            event_on_instruction: false,
            memory_op_actions: Vec::new(),
        };
        processor
            .map_iface(0xe000e000, Rc::new(RefCell::new(SystemControl::new())))
            .unwrap();
        processor
    }

    pub fn map(&mut self, address: u32, data: &[u8]) -> Result<(), ()> {
        self.map_iface(
            address,
            Rc::new(RefCell::new(RamMemory::new_from_slice(data))),
        )
    }

    pub fn map_iface(
        &mut self,
        address: u32,
        iface: Rc<RefCell<dyn MemoryInterface>>,
    ) -> Result<(), ()> {
        // TODO check overlap
        // TODO check len ovf
        let size = iface.borrow().size();
        self.memory_mappings.0.push(MemoryMap {
            address: address,
            size,
            iface: iface,
        });
        Ok(())
    }

    pub fn hook_code(&mut self, range: Range<usize>) {
        self.code_hooks.push(CodeHook { range: range })
    }

    /// Returns bytes at given address in memory
    pub fn u8_at(&mut self, address: u32) -> Result<u8, RunError> {
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemRead {
                address,
                size: 1,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let mut env = Env::new(self.cycles);
        let read = mapping
            .iface
            .borrow_mut()
            .read_u8(address - mapping.address, &mut env);
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
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemWrite {
                address,
                size: 1,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let mut env = Env::new(self.cycles);
        match mapping
            .iface
            .borrow_mut()
            .write_u8(address - mapping.address, value, &mut env)
        {
            Ok(()) => Ok(()),
            Err(e) => Err(RunError::MemWrite {
                address: address,
                size: 1,
                cause: e,
            }),
        }
    }

    pub fn set_u16le_at(&mut self, address: u32, value: u16) -> Result<(), RunError> {
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemWrite {
                address,
                size: 2,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let mut env = Env::new(self.cycles);
        match mapping
            .iface
            .borrow_mut()
            .write_u16le(address - mapping.address, value, &mut env)
        {
            Ok(()) => Ok(()),
            Err(e) => Err(RunError::MemWrite {
                address: address,
                size: 2,
                cause: e,
            }),
        }
    }

    /// Returns halfword at given address in memory
    pub fn u16le_at(&mut self, address: u32) -> Result<u16, RunError> {
        if let Some(mapping) = self.memory_mappings.get_mut(address) {
            let mut env = Env::new(self.cycles);
            let read = mapping
                .iface
                .borrow_mut()
                .read_u16le(address - mapping.address, &mut env);
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
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemRead {
                address,
                size: 4,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let mut env = Env::new(self.cycles);
        match mapping
            .iface
            .borrow_mut()
            .read_u32le(address - mapping.address, &mut env)
        {
            Ok(val) => Ok(val),
            Err(e) => Err(RunError::MemRead {
                address: address,
                size: 4,
                cause: e,
            }),
        }
    }

    pub fn set_u32le_at(&mut self, address: u32, value: u32) -> Result<(), RunError> {
        let mapping = self
            .memory_mappings
            .get_mut(address)
            .ok_or(RunError::MemWrite {
                address,
                size: 4,
                cause: MemoryAccessError::InvalidAddress,
            })?;
        let mut env = Env::new(self.cycles);
        mapping
            .iface
            .borrow_mut()
            .write_u32le(address - mapping.address, value, &mut env)
            .map_err(|e| RunError::MemWrite {
                address,
                size: 4,
                cause: e,
            })
    }

    /// Returns current value of the Stack Pointer (r13)
    pub fn sp(&self) -> u32 {
        self.registers[13].0
    }

    /// Sets value of the Stack Pointer (r13)
    ///
    /// # Arguments
    ///
    /// * `value` - New Stack Pointer value
    pub fn set_sp(&mut self, value: u32) {
        self.registers[13].0 = value
    }

    /// Returns current value of the Link Register (r14)
    pub fn lr(&self) -> u32 {
        self.registers[14].0
    }

    /// Sets Link Register (r14) value
    ///
    /// # Arguments
    ///
    /// * `value` - New Link Register value
    pub fn set_lr(&mut self, value: u32) {
        self.registers[14].0 = value
    }

    /// Returns current value of the Program Counter (r15)
    pub fn pc(&self) -> u32 {
        self.registers[15].0
    }

    /// Sets Program Counter (r15) value
    ///
    /// # Arguments
    ///
    /// * `value` - New Program Counter value
    pub fn set_pc(&mut self, value: u32) {
        self.registers[15].0 = value;
    }

    fn decode_instruction(
        &mut self,
        address: u32,
    ) -> Result<(InstructionBox, InstructionSize), RunError> {
        let hw = self.u16le_at(address)?;
        let (ins, size) = match thumb_ins_size(hw) {
            InstructionSize::Ins16 => (
                self.instruction_decoder.try_decode(
                    hw as u32,
                    InstructionSize::Ins16,
                    self.it_state.clone(),
                )?,
                InstructionSize::Ins16,
            ),
            InstructionSize::Ins32 => {
                let hw2 = self.u16le_at(address + 2)?;
                (
                    self.instruction_decoder.try_decode(
                        ((hw as u32) << 16) + hw2 as u32,
                        InstructionSize::Ins32,
                        self.it_state.clone(),
                    )?,
                    InstructionSize::Ins32,
                )
            }
        };
        Ok((ins, size))
    }

    /// Call `update` on memory mapping which requested an update during a previous operation.
    pub fn update_peripherals(&mut self) -> Option<Event> {
        let mut env = Env::new(self.cycles);
        for mapping in self.memory_mappings.0.iter_mut() {
            mapping.iface.borrow_mut().update(&mut env);
        }
        None
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
            let condition = ins
                .condition()
                .or(self.it_state.current_condition())
                .unwrap_or(Condition::Always);
            // Advance IT block state. This must be done before executing the instruction,
            // because if the current instruction is IT, it will load a new state.
            self.it_state.advance();

            let mut jump = false;
            if self.registers.apsr.test(condition) {
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
                    MemoryOpAction::Update(_) => panic!(), // This should be filtered prior
                }
            }
            self.cycles += 1;
            Ok(Event::Instruction { ins })
        }
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
        if self.registers.handler_mode {
            true
        } else {
            !self.registers.control.privileged_bit()
        }
    }

    /// Returns sum of values and update N, Z, C, and V flags in APSR.
    ///
    /// # Arguments
    ///
    /// * `x` - First sum operand
    /// * `y` - Second sum operand
    /// * `carry_in` - Input carry bit
    pub fn add_with_carry(&mut self, x: u32, y: u32, carry_in: bool) -> u32 {
        let (result, carry, overflow) = add_with_carry(x, y, carry_in);
        self.registers
            .apsr
            .set_nz(result)
            .set_c(carry)
            .set_v(overflow);
        result
    }

    /// Write value to PC, with interworking for ARM only from ARMv7
    pub fn alu_write_pc(&mut self, address: u32) {
        self.blx_write_pc(address)
    }

    /// Write value to PC, with interworking
    pub fn blx_write_pc(&mut self, address: u32) {
        self.registers.epsr.set_t(address & 1 == 1);
        self.set_pc(address & 0xfffffffe)
    }

    /// Write value to PC, with interworking
    pub fn bx_write_pc(&mut self, address: u32) {
        if self.registers.handler_mode && (address >> 28 == 0xf) {
            todo!()
        } else {
            self.blx_write_pc(address)
        }
    }

    pub fn condition_passed(&self) -> bool {
        if let Some(condition) = self.it_state.current_condition() {
            self.registers.apsr.test(condition)
        } else {
            true
        }
    }
}

/// Indexing implemented for easier access to the registers.
impl Index<RegisterIndex> for Arm7Processor {
    type Output = Register;

    fn index(&self, index: RegisterIndex) -> &Self::Output {
        &self.registers[index]
    }
}

/// Indexing implemented for easier access to the registers.
impl IndexMut<RegisterIndex> for Arm7Processor {
    fn index_mut(&mut self, index: RegisterIndex) -> &mut Self::Output {
        &mut self.registers[index]
    }
}
