use crate::builtin::helper::{Action, Method};
use crate::vm::Vm;
use crate::vm::gc::GarbageCollector;

pub struct Values {
    iter: super::Iter,
}

impl Method for Values {
    type Parent = super::Map;

    fn new(parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        let mut this = Self {
            iter: parent.iter(),
        };
        if let Some((_, value)) = this.iter.next(parent) {
            (this, Action::Output(value))
        } else {
            (this, Action::Stop)
        }
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn after_output(&mut self, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        if let Some((_, value)) = self.iter.next(parent) {
            Action::Output(value)
        } else {
            Action::Stop
        }
    }
}
