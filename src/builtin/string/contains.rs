use memchr::memmem;

use crate::builtin::helper::{Action, Function, err, input_before_input, last_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct Contains {
    string: Option<StringRef>,
}

impl Function for Contains {
    const NAME: &str = "Contains";

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
        let Some(haystack) = input_before_input!(self.string = input.as_string_ref()) else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let Some(needle) = last_input!(input.as_string_ref()) else {
            err!(
                vm,
                "type error: {} String {}",
                Self::NAME,
                input.type_name()
            );
        };
        let result = memmem::find(haystack.bytes(), needle.bytes()).is_some();
        Action::Output(Value::from(result))
    }
}
