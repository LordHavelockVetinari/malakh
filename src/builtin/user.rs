use std::io::{self, BufRead, Write};

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct User;

impl Function for User {
    const NAME: &str = "User";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::OptionalInput)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let mut stdout = io::stdout().lock();
        if let Err(error) = input
            .write_to(&mut stdout)
            .and_then(|_| stdout.write_all(b"\n"))
        {
            err!(vm, "{}", error);
        }
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        let mut buf = Vec::new();
        if let Err(error) = io::stdin().lock().read_until(b'\n', &mut buf) {
            err!(vm, "{}", error);
        }
        if buf.is_empty() {
            return Action::OptionalInput;
        }
        if buf.last() == Some(&b'\n') {
            buf.pop();
            if buf.last() == Some(&b'\r') {
                buf.pop();
            }
        }
        Action::Output(Value::alloc_from(&buf[..], vm.gc_mut()))
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
