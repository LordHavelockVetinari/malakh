use std::ptr::NonNull;

use crate::builtin::helper::{Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Bytes {
    owner: Value,
    bytes: NonNull<[u8]>,
}

impl Function for Bytes {
    const NAME: &str = "Bytes";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            owner: Value::ZERO,
            bytes: NonNull::from(&[]),
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(self.owner);
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(s) = input.as_string_ref() else {
            todo!("String::Bytes did not get a string");
        };
        let bytes = s.bytes();
        let Some((&first, rest)) = bytes.split_first() else {
            return Action::Stop;
        };
        let first = Value::alloc_from(first as isize, vm.gc_mut());
        self.owner = input;
        self.bytes = NonNull::from(rest);
        Action::Output(first)
    }

    fn after_output(&mut self, vm: &mut Vm) -> Action {
        let bytes = unsafe { self.bytes.as_ref() };
        let Some((&first, rest)) = bytes.split_first() else {
            return Action::Stop;
        };
        let first = Value::alloc_from(first as isize, vm.gc_mut());
        self.bytes = NonNull::from(rest);
        Action::Output(first)
    }
}
