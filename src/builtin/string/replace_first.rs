use memchr::memmem;

use crate::builtin::helper::{Action, Function, err, input_before_input, last_input};
use crate::vm::gc::GarbageCollector;
use crate::vm::string::StringRef;
use crate::vm::{Value, Vm};

pub struct ReplaceFirst {
    haystack: Option<StringRef>,
    needle: Option<StringRef>,
}

fn replace_first(
    haystack_ref: StringRef,
    needle: &[u8],
    replacement: &[u8],
    gc: &mut GarbageCollector,
) -> StringRef {
    let haystack = haystack_ref.bytes();
    if let Some(index) = memmem::find(haystack, needle) {
        let mut result = Vec::with_capacity(haystack.len() - needle.len() + replacement.len());
        result.extend(&haystack[..index]);
        result.extend(replacement);
        result.extend(&haystack[index + needle.len()..]);
        StringRef::new(&result, gc)
    } else {
        haystack_ref
    }
}

impl Function for ReplaceFirst {
    const NAME: &str = "ReplaceFirst";

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
        let result = replace_first(haystack, needle.bytes(), replacement.bytes(), vm.gc_mut());
        Action::Output(Value::from(result))
    }
}
