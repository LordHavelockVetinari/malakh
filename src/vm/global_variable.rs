use std::cell::Cell;

use crate::vm::user_process::{UserProcessFamily, UserProcessRef};
use crate::vm::{Value, Vm};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GlobalVariableState {
    Uninitialized,
    Initializing,
    Initialized(Value),
    Poisoned,
}

#[derive(Debug)]
pub struct GlobalVariable {
    initializer_family: &'static UserProcessFamily,
    state: Cell<GlobalVariableState>,
}

impl GlobalVariable {
    pub fn new(initializer_family: &'static UserProcessFamily) -> Self {
        Self {
            initializer_family,
            state: Cell::new(GlobalVariableState::Uninitialized),
        }
    }

    pub fn state(&self) -> GlobalVariableState {
        self.state.get()
    }

    pub fn make_initializer(&self, vm: &mut Vm) -> UserProcessRef {
        use GlobalVariableState::*;
        debug_assert_eq!(self.state.get(), Uninitialized);
        self.state.set(Initializing);
        UserProcessRef::new(self.initializer_family, &mut vm.gc, &[])
    }

    pub fn finish_init(&self, value: Value) {
        use GlobalVariableState::*;
        debug_assert_eq!(self.state.get(), Initializing);
        self.state.set(Initialized(value));
    }

    pub fn poison(&self) {
        use GlobalVariableState::*;
        debug_assert_eq!(self.state.get(), Initializing);
        self.state.set(Poisoned);
    }

    pub fn get(&self) -> Option<Value> {
        use GlobalVariableState::*;
        match self.state.get() {
            Initialized(x) => Some(x),
            _ => None,
        }
    }
}
