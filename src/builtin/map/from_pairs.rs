use crate::builtin::helper::{self, Action};
use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::gc::GarbageCollector;
use crate::vm::value::hashable::HashableValue;
use crate::vm::{Value, Vm};

pub struct FromPairs {
    result: BuiltinProcessRef,
    current_key: Option<HashableValue>,
}

impl FromPairs {
    fn get_mut(&mut self) -> &mut super::Map {
        unsafe { self.result.data_mut::<super::Map>() }
    }
}

impl helper::Function for FromPairs {
    const NAME: &str = "FromPairs";

    fn new(vm: &mut Vm) -> (Self, Action) {
        let index = *super::FAMILY_INDEX
            .get()
            .expect("Map index should have been initialized");
        let family = vm.get_builtin_family(index);
        let result = BuiltinProcessRef::new(family, None, vm);
        let this = Self {
            result,
            current_key: None,
        };
        (this, Action::OptionalInput)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(Value::from(self.result));
        if let Some(key) = self.current_key {
            gc.mark(key.get());
        }
    }

    fn input(&mut self, input: Value, _vm: &mut Vm) -> Action {
        match self.current_key {
            None => {
                let Some(key) = HashableValue::new(input) else {
                    todo!("non-hashable key");
                };
                self.current_key = Some(key);
                Action::Input
            }
            Some(key) => {
                self.get_mut().entry(key).insert((key, input));
                self.current_key = None;
                Action::OptionalInput
            }
        }
    }

    fn no_input(&mut self, _vm: &mut Vm) -> Action {
        Action::Output(Value::from(self.result))
    }
}
