use std::ptr::NonNull;

use memchr::memmem;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct FindFromEnd {
    haystack_ref: Option<StringRef>,
    needle_ref: Option<StringRef>,
    haystack: NonNull<[u8]>,
    needle: NonNull<[u8]>,
}

impl Function for FindFromEnd {
    const NAME: &str = "FindFromEnd";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            haystack_ref: None,
            needle_ref: None,
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
            self.haystack = NonNull::from(haystack);
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
            self.needle = NonNull::from(needle);
            Action::OptionalInput
        } else {
            err!(vm, "{} was not expecting input", Self::NAME);
        }
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        let haystack = unsafe { self.haystack.as_ref() };
        let needle = unsafe { self.needle.as_ref() };
        if needle.is_empty() {
            let result = Value::alloc_from(haystack.len(), vm.gc_mut());
            if let Some((_, without_last)) = haystack.split_last() {
                self.haystack = NonNull::from(without_last);
            } else {
                self.needle = NonNull::from(b"\x00" as &'static [u8]);
            }
            Action::Output(result)
        } else if let Some(index) = memmem::rfind(haystack, needle) {
            self.haystack = NonNull::from(&haystack[..index]);
            Action::Output(Value::alloc_from(index, vm.gc_mut()))
        } else {
            Action::Stop
        }
    }

    fn after_output(&mut self, _vm: &mut Vm) -> Action {
        Action::OptionalInput
    }
}
