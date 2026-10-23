use crate::builtin::helper::{Action, Function, err, input_before_optional_input, last_input};
use crate::util;
use crate::util::string::Utf8Error;
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct TrimStart {
    string: Option<StringRef>,
}

impl Function for TrimStart {
    const NAME: &str = "TrimStart";

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
        let Some(string) = input_before_optional_input!(self.string = input.as_string_ref()) else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let Some(pattern) = last_input!(input.as_string_ref()) else {
            err!(
                vm,
                "type error: {} String {}",
                Self::NAME,
                input.type_name()
            );
        };
        let pattern = pattern.bytes();
        if pattern.is_empty() {
            return Action::Output(Value::from(string));
        }
        let mut result = string.bytes();
        while let Some(s) = result.strip_prefix(pattern) {
            result = s;
        }
        Action::Output(Value::from(string.slice_raw(result, vm.gc_mut())))
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        let string = self
            .string
            .expect("process must have received at least one input");
        let bytes = string.bytes();
        match util::string::trim_whitespace_start(bytes) {
            Ok(result) => Action::Output(Value::alloc_from(result, vm.gc_mut())),
            Err(Utf8Error) => err!(vm, "invalid UTF-8 encoding"),
        }
    }
}
