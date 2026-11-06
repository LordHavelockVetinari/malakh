use std::io::Write;
use std::mem;

use crate::builtin::helper::{Action, Function, err, input_before_optional_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::string::writer::StringBuffer;
use crate::vm::{Value, Vm};

pub struct Join {
    buffer: StringBuffer,
    separator: Option<StringRef>,
    is_first_part: bool,
}

impl Function for Join {
    const NAME: &str = "Join";

    fn new(vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            buffer: StringBuffer::new(vm.gc_mut()),
            separator: None,
            is_first_part: true,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        self.buffer.gc_mark_content(gc);
        if let Some(separator) = self.separator {
            separator.gc_mark();
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(separator) = input_before_optional_input!(self.separator = input.as_string_ref()) else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let mut writer = self.buffer.writer(vm.gc_mut());
        if !mem::take(&mut self.is_first_part) {
            writer.write_all(separator.bytes()).unwrap();
        }
        input.write_to(&mut writer).unwrap();
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        let result = self.buffer.to_string(vm.gc_mut());
        Action::Output(Value::from(result))
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
