use std::ptr::NonNull;

use memchr::memmem;

use crate::builtin::helper::{Action, Function, err, input_before_input, last_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};
use crate::vm::string::StringRef;

pub struct Split {
    string: Option<StringRef>,
    separator: Option<StringRef>,
    bytes: Option<NonNull<[u8]>>,
}

impl Split {
    fn next(&mut self, gc: &mut GarbageCollector) -> Option<StringRef> {
        let bytes = unsafe { self.bytes?.as_ref() };
        let separator = self.separator.as_ref().unwrap().bytes();
        let substr = if let Some(index) = memmem::find(bytes, separator) {
            self.bytes = Some(NonNull::from(&bytes[index + separator.len()..]));
            &bytes[..index]
        } else {
            self.bytes = None;
            bytes
        };
        Some(self.string.unwrap().slice_raw(substr, gc))
    }
}

impl Function for Split {
    const NAME: &str = "Split";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            string: None,
            separator: None,
            bytes: None,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {
        if let Some(string) = self.string {
            string.gc_mark();
        }
        if let Some(separator) = self.separator {
            separator.gc_mark();
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(string) = input_before_input!(self.string = input.as_string_ref()) else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let Some(separator) = last_input!(self.separator = input.as_string_ref()) else {
            err!(vm, "type error: {} String {}", Self::NAME, input.type_name());
        };
        if separator.is_empty() {
            err!(vm, "separator is empty");
        };
        if string.is_empty() {
            return Action::Stop;
        }
        self.bytes = Some(string.bytes_non_null());
        if let Some(output) = self.next(vm.gc_mut()) {
            Action::Output(Value::from(output))
        } else {
            Action::Stop
        }
    }

    fn after_output(&mut self, vm: &mut Vm) -> Action {
        if let Some(output) = self.next(vm.gc_mut()) {
            Action::Output(Value::from(output))
        } else {
            Action::Stop
        }
    }
}
