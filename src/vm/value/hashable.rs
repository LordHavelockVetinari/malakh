use std::hash::{DefaultHasher, Hash, Hasher};

use malachite::Integer;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;

use crate::vm::Value;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HashableValue(Value);

fn hash<T: Hash + ?Sized>(x: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

impl HashableValue {
    pub fn new(value: Value) -> Option<Self> {
        if value.is_capture() || value.is_nan() {
            return None;
        }
        Some(Self(value))
    }

    pub fn get(self) -> Value {
        self.0
    }

    pub fn hash(self) -> u64 {
        match self.0.tag() {
            Value::BUILTIN_PROCESS_TAG
            | Value::USER_PROCESS_TAG
            | Value::SMALL_INT_TAG
            | Value::SYMBOL_TAG => hash(&self.0.as_addr()),
            Value::BIG_INT_TAG => hash(self.0.as_integer().unwrap()),
            Value::FLOAT_TAG => {
                let x = self.0.as_f64().unwrap();
                debug_assert!(!x.is_nan());
                // If x is an integer, should have the same hash as [ToInt x].
                if x.fract() == 0.0 {
                    const LOWER_BOUND: f64 = isize::MIN as f64;
                    const UPPER_BOUND: f64 = (isize::MAX as u128 + 1) as f64;
                    if (LOWER_BOUND..UPPER_BOUND).contains(&x) {
                        let n = x as isize;
                        if let Some(n) = Value::from_small_int(n) {
                            hash(&n.as_addr())
                        } else {
                            let n = Integer::from(n);
                            hash(&n)
                        }
                    } else {
                        let (n, _) = Integer::rounding_from(x, RoundingMode::Exact);
                        hash(&n)
                    }
                } else {
                    debug_assert_ne!(x, 0.0); // 0.0 and -0.0 should have the same hash.
                    hash(&x.to_bits())
                }
            }
            Value::STRING_TAG => {
                let s = self.0.as_string_ref().unwrap();
                hash(s.bytes())
            }
            Value::CAPTURE_TAG | 8.. => unreachable!(),
        }
    }
}
