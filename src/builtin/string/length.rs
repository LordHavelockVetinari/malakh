use crate::builtin::helper::{self, BasicFunction, BasicFunctionResult};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Length;

impl BasicFunction for Length {
    const NAME: &str = "Length";

    fn new(_vm: &mut Vm) -> Self {
        Self
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, vm: &mut Vm) -> helper::BasicFunctionResult {
        let Some(s) = input.as_string_ref() else {
            todo!("String::Length did not get a string");
        };
        let len = Value::alloc_from(s.bytes().len(), vm.gc_mut());
        BasicFunctionResult::Output(len)
    }
}
