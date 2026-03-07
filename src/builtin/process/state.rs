use crate::builtin::helper::{self, BasicFunction, BasicFunctionResult};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::ProcessRef;
use crate::vm::{Value, Vm};

pub struct State;

impl BasicFunction for State {
    const NAME: &str = "State";

    fn new(_vm: &mut Vm) -> Self {
        Self
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, _vm: &mut Vm) -> helper::BasicFunctionResult {
        let Some(p) = input.as_any_process_ref() else {
            todo!("Process::State did not get a process");
        };
        BasicFunctionResult::Output(Value::from(p.state()))
    }
}
