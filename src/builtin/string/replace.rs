use memchr::memmem;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct Replace {
    haystack: Option<StringRef>,
    needle: Option<StringRef>,
}

fn replace(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut start = 0;
    for index in memmem::find_iter(haystack, needle) {
        result.extend(&haystack[start..index]);
        result.extend(replacement);
        start = index + needle.len();
    }
    result.extend(&haystack[start..]);
    result
}

impl Function for Replace {
    const NAME: &str = "Replace";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            haystack: None,
            needle: None,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {
        if let Some(haystack) = self.haystack {
            haystack.gc_mark();
        }
        if let Some(needle) = self.needle {
            needle.gc_mark();
        }
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(s) = input.as_string_ref() else {
            if self.haystack.is_none() {
                err!(vm, "type error: {} {}", Self::NAME, input.type_name());
            } else if self.needle.is_none() {
                err!(
                    vm,
                    "type error: {} String {}",
                    Self::NAME,
                    input.type_name()
                );
            } else {
                err!(
                    vm,
                    "type error: {} String String {}",
                    Self::NAME,
                    input.type_name()
                );
            }
        };
        match (self.haystack, self.needle) {
            (None, None) => {
                self.haystack = Some(s);
                Action::Input
            }
            (Some(_), None) => {
                self.needle = Some(s);
                Action::Input
            }
            (Some(haystack), Some(needle)) => {
                let haystack = haystack.bytes();
                let needle = needle.bytes();
                let replacement = s.bytes();
                let result = replace(haystack, needle, replacement);
                Action::Output(Value::alloc_from(result, vm.gc_mut()))
            }
            (None, Some(_)) => unreachable!(),
        }
    }
}
