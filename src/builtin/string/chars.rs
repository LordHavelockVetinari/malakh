use std::cmp::min;
use std::ptr::NonNull;

use crate::builtin::helper::{Action, Function, err};
use crate::vm::error::ErrorRef;
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

// self.0 is assumed to be valid for the lifetime of this struct.
struct CharsOrBytes(NonNull<[u8]>);

impl Iterator for CharsOrBytes {
    type Item = Result<char, u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = unsafe { self.0.as_ref() };
        let start = &bytes[..min(bytes.len(), char::MAX_LEN_UTF8)];
        let chunk = start.utf8_chunks().next()?;
        if let Some(c) = chunk.valid().chars().next() {
            self.0 = NonNull::from(&bytes[c.len_utf8()..]);
            return Some(Ok(c));
        }
        if let Some(&b) = chunk.invalid().first() {
            self.0 = NonNull::from(&bytes[1..]);
            return Some(Err(b));
        }
        None
    }
}

pub struct Chars {
    owner: Value,
    iterator: CharsOrBytes,
}

impl Chars {
    fn proceed(&mut self, vm: &mut Vm) -> Action {
        match self.iterator.next() {
            Some(Ok(c)) => Action::Output(Value::alloc_from(c as u32, vm.gc_mut())),
            Some(Err(b)) => {
                let mut error = ErrorRef::new_from_builtin_family(vm, Self::NAME);
                error.extend(Value::from(Symbol::ENCODING_ERROR));
                error.extend(Value::alloc_from(
                    "unexpected byte in UTF-8 string:",
                    vm.gc_mut(),
                ));
                error.extend(Value::from(b));
                Action::Error(error)
            }
            None => Action::Stop,
        }
    }
}

impl Function for Chars {
    const NAME: &str = "Chars";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            owner: Value::ZERO,
            iterator: CharsOrBytes(NonNull::from(b"")),
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        gc.mark(self.owner);
    }

    fn input(&mut self, input: Value, vm: &mut Vm) -> Action {
        let Some(s) = input.as_string_ref() else {
            err!(vm, "type error: {} {}", Self::NAME, input.type_name());
        };
        self.owner = input;
        self.iterator = CharsOrBytes(NonNull::from(s.bytes()));
        self.proceed(vm)
    }

    fn after_output(&mut self, vm: &mut Vm) -> Action {
        self.proceed(vm)
    }

    fn after_error(&mut self, vm: &mut Vm) -> Action {
        self.proceed(vm)
    }
}
