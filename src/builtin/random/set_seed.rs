use crate::builtin::helper::{Action, Method};
use crate::vm::gc::GarbageCollector;
use crate::vm::value::hashable::HashableValue;
use crate::vm::{Value, Vm};

pub struct SetSeed;

impl Method for SetSeed {
    type Parent = super::Random;

    fn new(_parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        (Self, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        let seed = if let Some(seed) = input.int_to_u64() {
            seed
        } else if let Some(value) = HashableValue::new(input) {
            value.hash()
        } else {
            0
        };
        parent.rng.seed(seed);
        Action::Stop
    }
}
