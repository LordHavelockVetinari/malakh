use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::{ProcessRef, ProcessState};
use crate::vm::{Value, Vm};

pub struct Peek;

impl Function for Peek {
    const NAME: &str = "Peek";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(process) = input.as_any_process_ref() else {
            err!(vm, "type error: {} {:?}", Self::NAME, input);
        };
        if process.state() != ProcessState::Out {
            return Action::Stop;
        }
        Action::Output(process.output_slot())
    }
}
