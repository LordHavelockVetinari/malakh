use crate::builtin::helper::{self, Action, err};
use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::value::hashable::HashableValue;
use crate::vm::{Value, Vm};

enum Mode {
    ExpectKey,
    ExpectIs(HashableValue),
    ExpectValue(HashableValue),
}

pub struct Of {
    result: BuiltinProcessRef,
    mode: Mode,
}

impl Of {
    fn get_mut(&mut self) -> &mut super::Map {
        unsafe { self.result.data_mut::<super::Map>() }
    }
}

impl helper::Function for Of {
    const NAME: &str = "Of";

    fn new(vm: &mut Vm) -> (Self, Action) {
        let index = *super::FAMILY_INDEX
            .get()
            .expect("Map index should have been initialized");
        let family = vm.get_builtin_family(index);
        let result = BuiltinProcessRef::new(family, None, vm);
        let this = Self {
            result,
            mode: Mode::ExpectKey,
        };
        (this, Action::OptionalInput)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(Value::from(self.result));
        match self.mode {
            Mode::ExpectKey => {}
            Mode::ExpectIs(key) | Mode::ExpectValue(key) => gc.mark(key.get()),
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        match self.mode {
            Mode::ExpectKey => {
                let Some(key) = HashableValue::new(input) else {
                    err!(vm, "non-hashable key: {:?}", input);
                };
                self.mode = Mode::ExpectIs(key);
                Action::Input
            }
            Mode::ExpectIs(key) => {
                if let Some(symbol) = input.as_symbol()
                    && symbol == Symbol::IS
                {
                    self.mode = Mode::ExpectValue(key);
                    Action::Input
                } else {
                    err!(vm, "expected symbol .Is");
                }
            }
            Mode::ExpectValue(key) => {
                self.get_mut().entry(key).insert((key, input));
                self.mode = Mode::ExpectKey;
                Action::OptionalInput
            }
        }
    }

    fn no_input(&mut self, _vm: &mut Vm) -> Action {
        Action::Output(Value::from(self.result))
    }
}
