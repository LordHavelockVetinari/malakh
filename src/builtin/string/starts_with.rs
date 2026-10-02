use crate::builtin::helper::{Action, Function, err};
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
        let Some(s) = input.as_string_ref() else {
            if self.string.is_none() {
                err!(vm, "type error: {} {}", Self::NAME, input.type_name());
            } else {
                err!(
                    vm,
                    "type error: {} String {}",
                    Self::NAME,
                    input.type_name()
                );
            }
        };
        if let Some(string) = self.string {
            let string = string.bytes();
            let prefix = s.bytes();
            Action::Output(Value::from(string.starts_with(prefix)))
        } else {
            self.string = Some(s);
            Action::Input
        }
    }
}
