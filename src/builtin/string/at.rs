use crate::builtin::helper::{Action, Function, err, input_before_input, last_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct At {
    string: Option<StringRef>,
}

impl Function for At {
    const NAME: &str = "At";

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
        let Some(&result) = string.bytes().get(n) else {
            return Action::Stop;
        };
        Action::Output(Value::from(result))
    }
}
