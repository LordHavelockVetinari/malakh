use std::io::Write;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::writer::StringBuffer;
use crate::vm::{Value, Vm};

pub struct FromChars {
    buffer: StringBuffer,
}

impl Function for FromChars {
    const NAME: &str = "FromChars";

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
        let Some(c) = input.int_to_u32().and_then(|n| char::try_from(n).ok()) else {
            if input.is_int() {
                err!(
                    vm,
                    tag = ENCODING_ERROR,
                    "invalid character code: {:?}",
                    input
                );
            }
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        self.buffer
            .writer(vm.gc_mut())
            .write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            .unwrap();
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        Action::Output(Value::from(self.buffer.to_string(vm.gc_mut())))
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
