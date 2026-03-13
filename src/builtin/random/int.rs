use malachite::base::num::basic::traits::{One, Zero};
use malachite::base::num::logic::traits::BitIterable;
use malachite::platform::Limb;
use malachite::{Integer, Natural};

use crate::builtin::helper::{Action, Method, err};
use crate::vm::gc::GarbageCollector;
use crate::vm::symbol::Symbol;
use crate::vm::{Value, Vm};

fn random_natural_to(limit: &Natural, rng: &mut fastrand::Rng) -> Natural {
    assert_ne!(*limit, 0);
    if *limit == Natural::ONE {
        return Natural::ZERO;
    }
    let num_limbs = limit.limb_count() as usize;
    debug_assert_ne!(num_limbs, 0);
    let num_bits = limit.bits().len();
    let extra_bits = (num_bits % Limb::BITS as usize) as u32;
    let last_limb_mask = 1u64.checked_shl(extra_bits).unwrap_or(0).wrapping_sub(1);
    loop {
        let mut limbs = vec![0; num_limbs];
        rng.fill(bytemuck::cast_slice_mut::<u64, u8>(&mut limbs));
        *limbs.last_mut().unwrap() &= last_limb_mask;
        let candidate = Natural::from_owned_limbs_asc(limbs);
        if candidate < *limit {
            return candidate;
        }
    }
}

fn random_int_exclusive(
    rng: &mut fastrand::Rng,
    start: &Integer,
    end: &Integer,
    gc: &mut GarbageCollector,
) -> Value {
    debug_assert!(start < end);
    if let Ok(start) = i64::try_from(start)
        && let Ok(end) = i64::try_from(end)
    {
        Value::alloc_from(rng.i64(start..end), gc)
    } else {
        let delta = Natural::try_from(end - start).unwrap();
        let random = Integer::from(random_natural_to(&delta, rng));
        Value::alloc_from(start + random, gc)
    }
}

#[derive(PartialEq, Eq, Default)]
enum Mode {
    #[default]
    ExpectStart,
    ExpectCommand,
    ExpectExclusiveEnd,
    ExpectInclusiveEnd,
}

#[derive(Default)]
pub struct Int {
    mode: Mode,
    start: Integer,
}

impl Method for Int {
    type Parent = super::Random;

    fn new(_parent: &mut Self::Parent, _vm: &mut Vm) -> (Self, Action) {
        let this = Self {
            mode: Mode::ExpectStart,
            start: Integer::default(),
        };
        (this, Action::Input)
    }

    fn gc_mark_content(&self, _gc: &mut GarbageCollector) {}

    fn input(&mut self, input: Value, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        match self.mode {
            Mode::ExpectStart => {
                let Some(start) = input.int_to_integer() else {
                    err!(vm, "type error: <random> .Int {}", input.type_name());
                };
                self.start = start;
                self.mode = Mode::ExpectCommand;
                Action::OptionalInput
            }
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
                let Some(mut end) = input.int_to_integer() else {
                    err!(
                        vm,
                        "type error: <random> .Int Int .To {}",
                        input.type_name()
                    );
                };
                if self.mode == Mode::ExpectInclusiveEnd {
                    end += Integer::ONE;
                }
                if end <= self.start {
                    err!(vm, "range is empty");
                }
                let result = random_int_exclusive(&mut parent.rng, &self.start, &end, vm.gc_mut());
                Action::Output(result)
            }
        }
    }

    fn no_input(&mut self, parent: &mut Self::Parent, vm: &mut Vm) -> Action {
        let end = &self.start;
        let result = random_int_exclusive(&mut parent.rng, &Integer::ZERO, end, vm.gc_mut());
        Action::Output(result)
    }
}
