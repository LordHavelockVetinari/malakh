use crate::builtin::helper::{Action, Function, err, input_before_input, last_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct StartsWith {
    string: Option<StringRef>,
}

impl Function for StartsWith {
    const NAME: &str = "StartsWith";

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
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let Some(prefix) = last_input!(input.as_string_ref()) else {
            err!(
                vm,
                "type error: {} String {}",
                Self::NAME,
                input.type_name()
            );
        };
        let result = string.bytes().starts_with(prefix.bytes());
        Action::Output(Value::from(result))
    }
}
