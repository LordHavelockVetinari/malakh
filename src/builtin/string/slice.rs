use crate::builtin::helper::{self, Action};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

#[derive(PartialEq, Eq)]
enum Command {
    String,
    Keyword,
    Start,
    End,
}

pub struct Slice {
    string: Option<Value>,
    start: Option<usize>,
    end: Option<usize>,
    is_inclusive: bool,
    command: Command,
}

impl helper::Function for Slice {
    const NAME: &str = "Slice";

    fn new(_vm: &mut Vm) -> (Self, Action) {
        let this = Slice {
            string: None,
            start: None,
            end: None,
            is_inclusive: false,
            command: Command::String,
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, gc: &mut GarbageCollector) {
        if let Some(s) = self.string {
            gc.mark(s);
        }
    }

    fn input(&mut self, input: Value, _vm: &mut Vm) -> Action {
        match self.command {
            Command::String => {
                if !input.is_string() {
                    todo!("slice didn't get a string");
                }
                self.string = Some(input);
                self.command = Command::Keyword;
                Action::Input
            }
            Command::Keyword => {
                let Some(input) = input.as_symbol() else {
                    todo!("String::Slice expected a symbol");
                };
                if input == Symbol::FROM {
                    if self.start.is_some() {
                        todo!("slice got start twice");
                    }
                    self.command = Command::Start;
                } else if input == Symbol::TO {
                    if self.end.is_some() {
                        todo!("slice got end twice");
                    }
                    self.command = Command::End;
                } else if input == Symbol::THROUGH {
                    if self.end.is_some() {
                        todo!("slice got end twice");
                    }
                    self.is_inclusive = true;
                    self.command = Command::End;
                } else {
                    todo!("bad keyword passed to slice");
                }
                Action::Input
            }
            Command::Start => {
                let Some(n) = input.as_usize_saturating() else {
                    todo!("slice didn't get an integer");
                };
                self.start = Some(n);
                self.command = Command::Keyword;
                Action::OptionalInput
            }
            Command::End => {
                let Some(n) = input.as_usize_saturating() else {
                    todo!("slice didn't get an integer");
                };
                self.end = Some(n);
                self.command = Command::Keyword;
                Action::OptionalInput
            }
        }
    }

    fn no_input(&mut self, vm: &mut Vm) -> Action {
        if self.command != Command::Keyword {
            todo!("slice expected more input");
        }
        if self.start.is_none() && self.end.is_none() {
            todo!("slice must get either start or end");
        }
        let s = self
            .string
            .expect("slice should have gotten a string at this point")
            .as_string_ref()
            .expect("slice should check its string input");
        let bytes = s.bytes();
        let start = self.start.unwrap_or(0).min(bytes.len());
        let end = self
            .end
            .unwrap_or(usize::MAX)
            .saturating_add(self.is_inclusive as usize)
            .min(bytes.len())
            .max(start);
        let slice = s
            .slice(start, end - start, vm.gc_mut())
            .expect("slicing shouldn't fail");
        Action::Output(Value::from(slice))
    }
}
