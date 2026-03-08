use crate::builtin::helper::{self, Action, Function, err};
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

    fn input(&mut self, input: Value, vm: &mut Vm) -> helper::Action {
        let Some(proc) = input.as_any_process_ref() else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        if proc.state() != ProcessState::Stop {
            if let Some(error) = proc.error(vm) {
                err!(vm, cause = error);
            }
            err!(
                vm,
                "assertion error: process in {} state; expected .Stop state",
                proc.state(),
            );
        }
        Action::Stop
    }
}
