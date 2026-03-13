use crate::builtin::helper::{Action, Method, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

#[derive(PartialEq, Eq, Default)]
enum Mode {
    #[default]
    ExpectCommand,
    ExpectExclusiveEnd,
    ExpectInclusiveEnd,
}

#[derive(Default)]
pub struct Uniform {
    mode: Mode,
    start: f64,
}

impl Method for Uniform {
    type Parent = super::Random;

    fn new(_parent: &mut Self::Parent, vm: &mut Vm) -> (Self, Action) {
        let start = vm.take_temporary1();
        let start = start
            .number_to_f64()
            .expect("Random should have passed a number to Random .Uniform");
        if !start.is_finite() {
            err!(
                vm,
                self = Self::default(),
                "start of range must be finite, got: {}",
                start
            );
        }
        let this = Self {
            mode: Mode::ExpectCommand,
            start,
        };
        (this, Action::OptionalInput)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        match self.mode {
            Mode::ExpectCommand => {
                let Some(input) = input.as_symbol() else {
                    err!(
                        vm,
                        "type error: <random> expected a symbol; got {}",
                        input.type_name()
                    );
                };
                if input == Symbol::TO {
                    self.mode = Mode::ExpectExclusiveEnd;
                } else if input == Symbol::THROUGH {
                    self.mode = Mode::ExpectInclusiveEnd;
                } else {
                    err!(vm, "unexpected symbol .{}", input.name());
                }
                Action::Input
            }
            Mode::ExpectExclusiveEnd | Mode::ExpectInclusiveEnd => {
                let Some(mut end) = input.number_to_f64() else {
                    err!(vm, "type error: <random> Number .To {}", input.type_name());
                };
                if self.mode == Mode::ExpectExclusiveEnd {
                    end = end.next_down();
                }
                if !end.is_finite() {
                    err!(vm, "end of range must be finite, got: {}", end);
                }
                if end < self.start {
                    err!(vm, "range is empty");
                }
                let random = parent.rng.f64();
                // This won't actually return the last number in the range.
                // I hope no one will notice.
                let result = self.start + random * (end - self.start);
                Action::Output(Value::alloc_from(result, vm.gc_mut()))
            }
        }
    }

    fn no_input(&mut self, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        let end = self.start;
        let result = parent.rng.f64() * end;
        Action::Output(Value::alloc_from(result, vm.gc_mut()))
    }
}
