use std::io::{self, BufRead, Write};

use crate::builtin::helper;
use crate::vm::gc::GarbageCollector;
use crate::vm::{Value, Vm};

pub struct User;

impl helper::BasicAggregator for User {
    const NAME: &str = "User";

    fn new(_vm: &mut Vm) -> Self {
        Self
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn get(&mut self, vm: &mut Vm) -> Option<Value> {
        let mut buf = Vec::new();
        if io::stdin().lock().read_until(b'\n', &mut buf).is_err() {
            todo!();
        }
        if buf.is_empty() {
            return None;
        }
        if buf.last() == Some(&b'\n') {
            buf.pop();
            if buf.last() == Some(&b'\r') {
                buf.pop();
            }
        }
        Some(Value::string_from_bytes(&buf, vm.gc_mut()))
    }

    fn put(&mut self, value: Value, _vm: &mut Vm) {
        let mut stdout = io::stdout().lock();
        if value.write_to(&mut stdout).is_err() {
            todo!();
        }
        if stdout.write_all(b"\n").is_err() {
            todo!();
        }
    }
}
