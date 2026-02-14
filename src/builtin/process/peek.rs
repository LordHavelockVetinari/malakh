use crate::builtin::helper::{self, BasicFunction, BasicFunctionResult};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::ProcessState;
use crate::vm::{Value, Vm};

pub struct Peek;

impl BasicFunction for Peek {
    const NAME: &str = "Peek";

    fn new(_vm: &mut Vm) -> Self {
        Self
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, _vm: &mut Vm) -> helper::BasicFunctionResult {
        let Some(process) = input.as_any_process_ref() else {
            todo!("Process::Peek did not get a process");
        };
        if process.state() != ProcessState::Out {
            return BasicFunctionResult::Stop;
        }
        BasicFunctionResult::Output(process.output_slot())
    }
}
