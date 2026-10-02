use memchr::memmem;

use crate::builtin::helper::{Action, Function, err, input_before_input, last_input};
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
        let Some(haystack) = input_before_input!(self.haystack = input.as_string_ref()) else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        let Some(needle) = input_before_input!(self.needle = input.as_string_ref()) else {
            err!(
                vm,
                "type error: {} String {}",
                Self::NAME,
                input.type_name()
            );
        };
        let Some(replacement) = last_input!(input.as_string_ref()) else {
            err!(
                vm,
                "type error: {} String String {}",
                Self::NAME,
                input.type_name()
            );
        };
        let result = replace(haystack.bytes(), needle.bytes(), replacement.bytes());
        Action::Output(Value::alloc_from(result, vm.gc_mut()))
    }
}
