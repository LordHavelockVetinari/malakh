use std::mem;

use crate::builtin::helper::{self, Action};
use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct Of {
    data: Vec<Value>,
}

impl helper::Function for Of {
    const NAME: &str = "Of";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self { data: Vec::new() };
        (this, Action::OptionalInput)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        for &value in &self.data {
            gc.mark(value);
        }
    }

    fn input(&mut self, input: Value, _vm: &mut Vm) -> Action {
        self.data.push(input);
        Action::OptionalInput
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        let list_index = *super::FAMILY_INDEX
            .get()
            .expect("list should be initialized");
        let list_family = vm.get_builtin_family(list_index);
        let mut list_ref = BuiltinProcessRef::new(list_family, None, vm);
        let list = unsafe { list_ref.data_mut::<super::List>() };
        debug_assert!(list.data.is_empty());
        list.data = mem::take(&mut self.data);
        Action::Output(Value::from(list_ref))
    }
}
