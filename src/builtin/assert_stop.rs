use crate::builtin::helper::{self, Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::{ProcessRef, ProcessState};
use crate::vm::{Value, Vm};

pub struct AssertStop;

impl Function for AssertStop {
    const NAME: &str = "AssertStop";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, _vm: &mut Vm) -> helper::Action {
        let Some(proc) = input.as_any_process_ref() else {
            todo!("expected a process");
        };
        if proc.state() != ProcessState::Stop {
            // On error, should re-raise the same error.
            todo!("state was not .Stop");
        }
        Action::Stop
    }
}
