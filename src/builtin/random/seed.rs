use crate::builtin::helper::{Action, Method};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Seed;

impl Method for Seed {
    type Parent = super::Random;

    fn new(parent: &mut Self::Parent, vm: &mut Vm) -> (Self, Action) {
        let seed = parent.rng.get_seed();
        (Self, Action::Output(Value::alloc_from(seed, vm.gc_mut())))
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}
}
