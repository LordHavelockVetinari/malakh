use either::Either::{Left, Right};

use crate::builtin::helper::{Action, Function, err, input_before_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct Repeat {
    string: Option<StringRef>,
}

impl Function for Repeat {
    const NAME: &str = "Repeat";

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
        let n = match input.as_int() {
            Some(Left(n)) => {
                let Ok(n) = usize::try_from(n) else {
                    err!(vm, "negative number of repetitions");
                };
                n
            }
            Some(Right(n)) => {
                let Ok(n) = usize::try_from(n) else {
                    if *n < 0 {
                        err!(vm, "negative number of repetitions");
                    } else if string.bytes().is_empty() {
                        return Action::Output(Value::from(string));
                    } else {
                        err!(vm, "too many repetitions");
                    }
                };
                n
            }
            None => err!(vm, "type error: {} {}", Self::NAME, input.type_name()),
        };
        let result = string.bytes().repeat(n);
        Action::Output(Value::alloc_from(result, vm.gc_mut()))
    }
}
