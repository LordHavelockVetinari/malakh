use crate::builtin::helper::{Action, Method};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Urn {
    data: Vec<Value>,
}

impl Method for Urn {
    type Parent = super::Random;

    fn new(_parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        let this = Self { data: Vec::new() };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        for &value in &self.data {
            gc.mark(value);
        }
    }

    fn input(&mut self, input: Value, _parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        self.data.push(input);
        Action::OptionalInput
    }

    fn no_input(&mut self, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        let i = parent.rng.usize(0..self.data.len());
        let value = self.data.swap_remove(i);
        Action::Output(value)
    }

    fn after_output(&mut self, _parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        if self.data.is_empty() {
            Action::Input
        } else {
            Action::OptionalInput
        }
    }
}
