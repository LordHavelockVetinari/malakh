use crate::builtin::helper::{Action, Method};
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Each {
    iter: super::Iter,
    next_value: Option<Value>,
}

impl Method for Each {
    type Parent = super::Map;

    fn new(parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        let mut this = Self {
            iter: parent.iter(),
            next_value: None,
        };
        if let Some((key, value)) = this.iter.next(parent) {
            this.next_value = Some(value);
            (this, Action::Output(key.get()))
        } else {
            (this, Action::Stop)
        }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(value) = self.next_value {
            gc.mark(value);
        }
    }

    fn after_output(&mut self, parent: &mut Self::Parent, _vm: &mut Vm) -> Action {
        if let Some(next) = self.next_value.take() {
            Action::Output(next)
        } else if let Some((key, value)) = self.iter.next(parent) {
            self.next_value = Some(value);
            Action::Output(key.get())
        } else {
            Action::Stop
        }
    }
}
