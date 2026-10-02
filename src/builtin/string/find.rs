use std::ptr::NonNull;

use memchr::memmem;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct Find {
    haystack_ref: Option<StringRef>,
    needle_ref: Option<StringRef>,
    offset: usize,
    haystack: NonNull<[u8]>,
    needle: NonNull<[u8]>,
}

impl Function for Find {
    const NAME: &str = "Find";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            haystack_ref: None,
            needle_ref: None,
            offset: 0,
            haystack: NonNull::from(b""),
            needle: NonNull::from(b""),
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {
        if let Some(haystack) = self.haystack_ref {
            haystack.gc_mark();
        }
        if let Some(needle) = self.needle_ref {
            needle.gc_mark();
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        if self.haystack_ref.is_none() {
            let Some(haystack) = input.as_string_ref() else {
                err!(vm, "type error: {} {}", Self::NAME, input.type_name());
            };
            self.haystack_ref = Some(haystack);
            self.haystack = haystack.bytes_non_null();
            Action::Input
        } else if self.needle_ref.is_none() {
            let Some(needle) = input.as_string_ref() else {
                err!(
                    vm,
                    "type error: {} String {}",
                    Self::NAME,
                    input.type_name()
                );
            };
            self.needle_ref = Some(needle);
            self.needle = needle.bytes_non_null();
            Action::OptionalInput
        } else {
            err!(vm, "{} was not expecting input", Self::NAME);
        }
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        let haystack = unsafe { self.haystack.as_ref() };
        let needle = unsafe { self.needle.as_ref() };
        if needle.is_empty() {
            if self.offset > haystack.len() {
                return Action::Stop;
            }
            let result = Value::alloc_from(self.offset, vm.gc_mut());
            self.offset += 1;
            Action::Output(result)
        } else if let Some(index) = memmem::find(&haystack[self.offset..], needle) {
            let true_index = index + self.offset;
            self.offset = true_index + needle.len();
            Action::Output(Value::alloc_from(true_index, vm.gc_mut()))
        } else {
            Action::Stop
        }
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
