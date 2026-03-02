use std::alloc::{self, Layout};
use std::mem::{self, MaybeUninit};
use std::ptr::NonNull;

use crate::vm::gc::GarbageCollector;
use crate::vm::process::ProcessState;

use super::gc::GcInfo;
use super::{Instruction, Value};

#[derive(Debug)]
pub struct UserProcessFamily {
    pub code: &'static [Instruction],
    pub memory_len: usize,
}

// A process consists of a process header followed by one or more Values,
// which are the process's memory.
// The first cell in memory also serves as the output slot.
#[repr(C, align(8))]
pub struct UserProcessHeader {
    gc_info: GcInfo,
    state: ProcessState,
    can_resume: bool,
    family: &'static UserProcessFamily,
    instruction_pointer: *const Instruction,
}

const _: () = assert!(size_of::<UserProcessHeader>() % size_of::<Value>() == 0);

impl UserProcessFamily {
    fn layout(&self) -> Layout {
        Layout::new::<UserProcessHeader>()
            .extend(Layout::array::<Value>(self.memory_len).unwrap())
            .unwrap()
            .0
    }
}

// SAFETY:
// The pointer must be valid throughout the struct's lifetime.
#[derive(Clone, Copy, Debug)]
pub struct UserProcessRef(pub NonNull<UserProcessHeader>);

impl UserProcessRef {
    fn header(&self) -> &UserProcessHeader {
        unsafe { self.0.as_ref() }
    }

    fn header_mut(&mut self) -> &mut UserProcessHeader {
        unsafe { self.0.as_mut() }
    }

    pub fn family(&self) -> &'static UserProcessFamily {
        self.header().family
    }

    fn memory_uninit(&mut self) -> *mut [MaybeUninit<Value>] {
        std::ptr::slice_from_raw_parts_mut(
            unsafe { self.0.add(1) }
                .cast::<MaybeUninit<Value>>()
                .as_ptr(),
            self.family().memory_len,
        )
    }

    pub fn memory(&self) -> &[Value] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.add(1).cast::<Value>().as_ptr(),
                self.family().memory_len,
            )
        }
    }

    pub fn memory_mut(&mut self) -> &mut [Value] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.0.add(1).cast::<Value>().as_ptr(),
                self.family().memory_len,
            )
        }
    }

    pub fn instruction_pointer(&self) -> *const Instruction {
        self.header().instruction_pointer
    }

    pub fn instruction_pointer_mut(&mut self) -> &mut *const Instruction {
        &mut self.header_mut().instruction_pointer
    }

    pub fn output_slot(&self) -> Value {
        self.memory()[0]
    }

    pub fn output_slot_mut(&mut self) -> &mut Value {
        &mut self.memory_mut()[0]
    }

    pub fn state(&self) -> ProcessState {
        self.header().state
    }

    pub fn state_mut(&mut self) -> &mut ProcessState {
        &mut self.header_mut().state
    }

    pub fn set_can_resume(&mut self) {
        self.header_mut().can_resume = true;
    }

    pub fn take_can_resume(&mut self) -> bool {
        mem::replace(&mut self.header_mut().can_resume, false)
    }

    pub fn new(family: &'static UserProcessFamily, gc: &mut GarbageCollector) -> Self {
        let layout = family.layout();
        let mut this = Self(
            NonNull::new(unsafe { alloc::alloc(layout) })
                .unwrap()
                .cast::<UserProcessHeader>(),
        );
        unsafe {
            this.0.write(UserProcessHeader {
                gc_info: GcInfo::default(),
                family,
                instruction_pointer: &family.code[0],
                state: ProcessState::Run,
                can_resume: false,
            });
            this.memory_uninit()
                .as_mut()
                .unwrap()
                .fill(MaybeUninit::new(Value::ZERO));
        }
        gc.start_tracking(Value::from(this), layout.size());
        this
    }

    pub fn gc_mark(&self, gc: &mut GarbageCollector) {
        if self.header().gc_info.mark() {
            return;
        }
        for &value in self.memory() {
            gc.mark(value);
        }
    }

    // True if the value is still alive.
    pub fn gc_sweep(&mut self) -> bool {
        if self.header().gc_info.unmark() {
            return true;
        }
        unsafe {
            alloc::dealloc(self.0.as_ptr().cast::<u8>(), self.family().layout());
        }
        false
    }
}
