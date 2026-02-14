use std::cell::OnceCell;

use crate::vm::user_process::{UserProcessFamily, UserProcessRef};
use crate::vm::{Value, Vm};

#[derive(Debug)]
pub struct GlobalVariable {
    initializer_family: Option<&'static UserProcessFamily>,
    value: OnceCell<Value>,
}

impl GlobalVariable {
    pub fn new(initializer_family: Option<&'static UserProcessFamily>) -> Self {
        Self {
            initializer_family,
            value: OnceCell::new(),
        }
    }

    pub fn begin_init(&self, vm: &mut Vm) {
        if self.value.get().is_none() {
            let Some(initializer_family) = self.initializer_family else {
                panic!("tried to initialize global variable with no initializer");
            };
            let initializer = UserProcessRef::new(initializer_family, &mut vm.gc);
            vm.enter_user_process(initializer);
        }
    }

    pub fn value(&self) -> &OnceCell<Value> {
        &self.value
    }
}
