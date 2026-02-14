use crate::builtin::helper::{Action, Method};
use crate::vm::Vm;
use crate::vm::gc::GarbageCollector;

pub struct Each {
    index: usize,
}

impl Method for Each {
    type Parent = super::List;

    fn new(parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        let this = Self { index: 1 };
        if let Some(&first) = parent.data.first() {
            (this, Action::Output(first))
        } else {
            (this, Action::Stop)
        }
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn after_output(&mut self, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        if let Some(&item) = parent.data.get(self.index) {
            self.index += 1;
            Action::Output(item)
        } else {
            Action::Stop
        }
    }
}
