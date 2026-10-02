use std::cmp::min;

use crate::builtin::helper::{Action, Function, err, input_before_input, last_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct CharAt {
    string: Option<StringRef>,
}

impl Function for CharAt {
    const NAME: &str = "CharAt";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self { string: None };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {
        if let Some(string) = self.string {
            string.gc_mark();
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(string) = input_before_input!(self.string = input.as_string_ref()) else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name())
        };
        let Some(n) = last_input!(input.as_usize()) else {
            if input.is_int() {
                return Action::Stop;
            }
            err!(
                vm,
                "type error: {} String {}",
                Self::NAME,
                input.type_name()
            )
        };
        let bytes = string.bytes();
        let Some(slice) = bytes.get(n..min(n.saturating_add(char::MAX_LEN_UTF8), bytes.len()))
        else {
            return Action::Stop;
        };
        if let Some(chunk) = slice.utf8_chunks().next()
            && let Some(c) = chunk.valid().chars().next()
        {
            Action::Output(Value::alloc_from(c.to_string(), vm.gc_mut()))
        } else {
            Action::Stop
        }
    }
}
