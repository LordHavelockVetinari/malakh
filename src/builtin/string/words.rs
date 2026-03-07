use std::ptr::NonNull;

use crate::builtin::helper::{Action, Function};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct Words {
    owner: Option<StringRef>,
    data: NonNull<str>,
}

impl Words {
    fn next(&mut self) -> Option<NonNull<[u8]>> {
        let s = unsafe { self.data.as_ref() };
        let result = s.split_whitespace().next()?.as_bytes();
        let endpoint = result.as_ptr_range().end;
        let skipped = unsafe { endpoint.offset_from_unsigned(s.as_ptr()) };
        self.data = NonNull::from(&s[skipped..]);
        Some(NonNull::from(result))
    }
}

impl Function for Words {
    const NAME: &str = "Words";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            owner: None,
            data: NonNull::from(""),
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(owner) = self.owner {
            gc.mark(Value::from(owner));
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(owner) = input.as_string_ref() else {
            todo!("String::Words did not get a string");
        };
        self.owner = Some(owner);
        let Ok(s) = str::from_utf8(owner.bytes()) else {
            todo!("String::Words did not get a valid UTF-8 string")
        };
        self.data = NonNull::from(s);
        match self.next() {
            Some(result) => {
                let result = unsafe { owner.slice_raw(result, vm.gc_mut()) };
                Action::Output(Value::from(result))
            }
            None => Action::Stop,
        }
    }

    fn after_output(&mut self, vm: &mut Vm) -> Action {
        let owner = self.owner.unwrap();
        match self.next() {
            Some(result) => {
                let result = unsafe { owner.slice_raw(result, vm.gc_mut()) };
                Action::Output(Value::from(result))
            }
            None => Action::Stop,
        }
    }
}
