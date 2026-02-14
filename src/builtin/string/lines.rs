use std::ptr::NonNull;

use crate::builtin::helper::{BasicFunction, BasicFunctionResult};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct Lines {
    owner: Option<StringRef>,
    data: NonNull<[u8]>,
}

impl Lines {
    fn next(&mut self) -> Option<NonNull<[u8]>> {
        let bytes = unsafe { self.data.as_ref() };
        if bytes.is_empty() {
            return None;
        }
        let endpoint = bytes
            .iter()
            .position(|&c| c == b'\n')
            .map(|pos| pos + 1)
            .unwrap_or(bytes.len());
        let (mut line, rest) = bytes.split_at(endpoint);
        self.data = NonNull::from(rest);
        if let Some((&b'\n', trimmed)) = line.split_last() {
            line = trimmed;
            if let Some((&b'\r', trimmed)) = line.split_last() {
                line = trimmed;
            }
        }
        Some(NonNull::from(line))
    }
}

impl BasicFunction for Lines {
    const NAME: &str = "Lines";

    fn new(_vm: &mut Vm) -> Self {
        Self {
            owner: None,
            data: NonNull::from(&[]),
        }
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(owner) = self.owner.clone() {
            gc.mark(Value::from_string_ref(owner));
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> BasicFunctionResult {
        let Some(owner) = input.as_string_ref() else {
            todo!("String::Words did not get a string");
        };
        self.owner = Some(owner.clone());
        self.data = NonNull::from(owner.bytes());
        match self.next() {
            Some(result) => {
                let result = unsafe { owner.slice_raw(result, vm.gc_mut()) };
                BasicFunctionResult::Output(Value::from_string_ref(result))
            }
            None => BasicFunctionResult::Stop,
        }
    }

    fn after_output(&mut self, vm: &mut Vm) -> BasicFunctionResult {
        let owner = self.owner.clone().unwrap();
        match self.next() {
            Some(result) => {
                let result = unsafe { owner.slice_raw(result, vm.gc_mut()) };
                BasicFunctionResult::Output(Value::from_string_ref(result))
            }
            None => BasicFunctionResult::Stop,
        }
    }
}
