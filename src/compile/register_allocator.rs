use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::compile::error::CompilationError;
use crate::parse::location::Location;

const MAX_NUM_REGISTERS: usize = u16::MAX as usize;

pub struct RegisterAllocator {
    // Including output slot.
    num_registers: usize,
    // Is the output slot (index 0) free.
    is_output_slot_free: bool,
    // Excluding the output slot (index 0).
    currently_free: BinaryHeap<Reverse<u16>>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            num_registers: 1,
            is_output_slot_free: true,
            currently_free: BinaryHeap::new(),
        }
    }

    pub fn alloc(&mut self, location: &Location) -> Result<u16, CompilationError> {
        if let Some(Reverse(index)) = self.currently_free.pop() {
            Ok(index)
        } else if self.num_registers == MAX_NUM_REGISTERS {
            CompilationError::err("process requires too much memory", location)
        } else {
            self.num_registers += 1;
            Ok((self.num_registers - 1) as u16)
        }
    }

    pub fn alloc_temporary(&mut self, location: &Location) -> Result<u16, CompilationError> {
        if self.is_output_slot_free {
            self.is_output_slot_free = false;
            Ok(0)
        } else {
            self.alloc(location)
        }
    }

    pub fn dealloc(&mut self, index: u16) {
        if index == 0 {
            self.is_output_slot_free = true;
        } else {
            self.currently_free.push(Reverse(index));
        }
    }

    pub fn are_all_freed(&self) -> bool {
        self.is_output_slot_free && self.num_registers == self.currently_free.len() + 1
    }

    pub fn required_num_registers(&self) -> usize {
        self.num_registers
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegisterChoice {
    // `Any` means: either choose an existing register with the correct value,
    // or allocate a new temporary register.
    Any,
    // `AllocNew` means: this value goes into a newly created variable.
    AllocNew,
    // `Existing` means: this value goes into an existing variable.
    Existing(u16),
}

impl RegisterChoice {
    pub fn or_alloc(
        self,
        allocator: &mut RegisterAllocator,
        location: &Location,
    ) -> Result<ChosenRegister, CompilationError> {
        match self {
            Self::Any => {
                let reg = allocator.alloc_temporary(location)?;
                Ok(ChosenRegister::owned(reg))
            }
            Self::AllocNew => {
                let reg = allocator.alloc(location)?;
                Ok(ChosenRegister::owned(reg))
            }
            Self::Existing(index) => Ok(ChosenRegister::shared(index)),
        }
    }

    pub fn use_existing_or_alloc(
        self,
        existing: u16,
        allocator: &mut RegisterAllocator,
        location: &Location,
    ) -> Result<ChosenRegister, CompilationError> {
        match self {
            Self::Any => Ok(ChosenRegister::shared(existing)),
            Self::AllocNew => {
                let reg = allocator.alloc(location)?;
                Ok(ChosenRegister::owned(reg))
            }
            Self::Existing(index) => Ok(ChosenRegister::shared(index)),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ChosenRegister {
    pub index: u16,
    pub is_owned: bool,
}

impl ChosenRegister {
    pub fn owned(index: u16) -> Self {
        Self {
            index,
            is_owned: true,
        }
    }

    pub fn shared(index: u16) -> Self {
        Self {
            index,
            is_owned: false,
        }
    }

    pub fn dealloc(self, allocator: &mut RegisterAllocator) {
        if self.is_owned {
            allocator.dealloc(self.index);
        }
    }
}
