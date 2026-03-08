use std::io::Write;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::writer::StringBuffer;
use crate::vm::{Value, Vm};

pub struct FromBytes {
    buffer: StringBuffer,
}

impl Function for FromBytes {
    const NAME: &str = "FromBytes";

    fn new(vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            buffer: StringBuffer::new(vm.gc_mut()),
        };
        (this, Action::OptionalInput)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        self.buffer.gc_mark_content(gc);
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(n) = input.as_small_int() else {
            if input.is_big_int() {
                err!(vm, "expected a byte");
            }
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let Ok(c) = u8::try_from(n) else {
            err!(vm, "expected a byte");
        };
        self.buffer.writer(vm.gc_mut()).write_all(&[c]).unwrap();
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        Action::Output(Value::from(self.buffer.to_string(vm.gc_mut())))
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
