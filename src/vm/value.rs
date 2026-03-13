use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::io;
use std::num::NonZero;
use std::ptr::{self, NonNull};

use either::Either::{self, Left, Right};
use malachite::Integer;
use malachite::base::num::arithmetic::traits::{Parity, Pow};
use malachite::base::num::basic::traits::Zero;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;

use crate::vm::big_int::{BigIntInner, BigIntRef};
use crate::vm::builtin_process::{BuiltinProcessHeader, BuiltinProcessRef};
use crate::vm::capture::{CaptureData, CaptureRef};
use crate::vm::error::{PowerError, TypeError};
use crate::vm::float::{FloatData, FloatRef};
use crate::vm::gc::GarbageCollector;
use crate::vm::process::AnyProcessRef;
use crate::vm::string::{StringHeader, StringRef};
use crate::vm::symbol::Symbol;
use crate::vm::user_process::{UserProcessHeader, UserProcessRef};

pub mod hashable;

#[derive(Clone, Copy)]
pub struct Value(pub NonNull<u8>);

const _: () = assert!(size_of::<Value>() == size_of::<Option<Value>>());

impl Value {
    pub const NUM_PTR_TAG_BITS: usize = 3;
    pub const PTR_TAG_MASK: usize = (1 << Self::NUM_PTR_TAG_BITS) - 1;
    pub const MIN_SMALL_INT: isize = isize::MIN >> Self::NUM_PTR_TAG_BITS;
    pub const MAX_SMALL_INT: isize = isize::MAX >> Self::NUM_PTR_TAG_BITS;

    pub const BUILTIN_PROCESS_TAG: usize = 0;
    pub const USER_PROCESS_TAG: usize = 1;
    pub const SMALL_INT_TAG: usize = 2;
    pub const BIG_INT_TAG: usize = 3;
    pub const FLOAT_TAG: usize = 4;
    pub const STRING_TAG: usize = 5;
    pub const SYMBOL_TAG: usize = 6;
    pub const CAPTURE_TAG: usize = 7;

    fn as_addr(self) -> usize {
        self.0.addr().get()
    }

    pub fn tag(self) -> usize {
        self.as_addr() & Self::PTR_TAG_MASK
    }

    pub fn type_name(self) -> &'static str {
        match self.tag() {
            Self::BUILTIN_PROCESS_TAG | Self::USER_PROCESS_TAG => "Process",
            Self::SMALL_INT_TAG | Self::BIG_INT_TAG => "Int",
            Self::FLOAT_TAG => "Float",
            Self::STRING_TAG => "String",
            Self::SYMBOL_TAG => "Symbol",
            Self::CAPTURE_TAG => self.as_capture_ref().unwrap().value().type_name(),
            _ => unreachable!(),
        }
    }

    pub fn is_small_int(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::SMALL_INT_TAG
    }

    pub fn is_big_int(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::BIG_INT_TAG
    }

    pub fn is_int(self) -> bool {
        self.is_small_int() || self.is_big_int()
    }

    pub fn is_float(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::FLOAT_TAG
    }

    pub fn is_number(self) -> bool {
        self.is_int() || self.is_float()
    }

    pub fn is_nan(self) -> bool {
        self.as_f64().is_some_and(f64::is_nan)
    }

    pub fn is_user_process(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::USER_PROCESS_TAG
    }

    pub fn is_builtin_process(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::BUILTIN_PROCESS_TAG
    }

    pub fn is_process(self) -> bool {
        self.is_builtin_process() || self.is_user_process()
    }

    pub fn is_symbol(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::SYMBOL_TAG
    }

    pub fn is_string(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::STRING_TAG
    }

    pub fn is_capture(self) -> bool {
        self.as_addr() & Self::PTR_TAG_MASK == Self::CAPTURE_TAG
    }

    pub const ZERO: Self = Self::from_small_int(0).unwrap();
    pub const ONE: Self = Self::from_small_int(1).unwrap();
    pub const FALSE: Self = Self(unsafe {
        NonNull::new_unchecked((&raw const *Symbol::FALSE as *mut u8).add(Self::SYMBOL_TAG))
    });
    pub const TRUE: Self = Self(unsafe {
        NonNull::new_unchecked((&raw const *Symbol::TRUE as *mut u8).add(Self::SYMBOL_TAG))
    });
    pub const UNDEFINED: Self = Self(unsafe {
        NonNull::new_unchecked((&raw const *Symbol::UNDEFINED as *mut u8).add(Self::SYMBOL_TAG))
    });

    pub fn is_bool(self) -> bool {
        self.0 == Self::FALSE.0 || self.0 == Self::TRUE.0
    }

    pub const fn from_small_int(n: isize) -> Option<Self> {
        if n < Self::MIN_SMALL_INT || n > Self::MAX_SMALL_INT {
            return None;
        }
        const {
            assert!(Self::SMALL_INT_TAG != 0);
        }
        let n = (n << Self::NUM_PTR_TAG_BITS) as usize | Self::SMALL_INT_TAG;
        Some(Value(unsafe {
            NonNull::new(ptr::without_provenance_mut(n)).unwrap_unchecked()
        }))
    }

    pub fn alloc_from<T: AllocIntoValue>(x: T, gc: &mut GarbageCollector) -> Self {
        x.alloc_into_value(gc)
    }

    pub fn as_small_int(self) -> Option<isize> {
        let addr = self.as_addr();
        if !self.is_small_int() {
            return None;
        }
        Some((addr as isize) >> Self::NUM_PTR_TAG_BITS)
    }

    pub fn as_big_int_ref(self) -> Option<BigIntRef> {
        if !self.is_big_int() {
            return None;
        }
        Some(BigIntRef(
            self.0
                .map_addr(|addr| unsafe {
                    NonZero::new(addr.get() & !Self::PTR_TAG_MASK).unwrap_unchecked()
                })
                .cast::<BigIntInner>(),
        ))
    }

    pub fn as_integer(&self) -> Option<&Integer> {
        self.as_big_int_ref()
            .map(|i| unsafe { i.value().as_ref().unwrap_unchecked() })
    }

    pub fn as_int(&self) -> Option<Either<isize, &Integer>> {
        if let Some(n) = self.as_small_int() {
            Some(Left(n))
        } else if let Some(n) = self.as_integer() {
            Some(Right(n))
        } else {
            None
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self.as_int() {
            None => None,
            Some(Left(n)) => usize::try_from(n).ok(),
            Some(Right(n)) => usize::try_from(n).ok(),
        }
    }

    pub fn as_usize_saturating(&self) -> Option<usize> {
        if let Some(n) = self.as_usize() {
            return Some(n);
        }
        match self.as_int()? {
            Left(n) => {
                debug_assert!(n < 0);
                Some(0)
            }
            Right(n) => {
                if *n < 0 {
                    Some(0)
                } else {
                    debug_assert!(*n > usize::MAX);
                    Some(usize::MAX)
                }
            }
        }
    }

    pub fn as_float_ref(&self) -> Option<FloatRef> {
        if !self.is_float() {
            return None;
        }
        Some(FloatRef(
            self.0
                .map_addr(|addr| unsafe {
                    NonZero::new(addr.get() & !Self::PTR_TAG_MASK).unwrap_unchecked()
                })
                .cast::<FloatData>(),
        ))
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.as_float_ref().map(|f| f.value())
    }

    pub fn as_capture_ref(&self) -> Option<CaptureRef> {
        if !self.is_capture() {
            return None;
        }
        Some(CaptureRef(
            self.0
                .map_addr(|addr| unsafe {
                    NonZero::new(addr.get() & !Self::PTR_TAG_MASK).unwrap_unchecked()
                })
                .cast::<CaptureData>(),
        ))
    }

    pub fn as_user_process_ref(self) -> Option<UserProcessRef> {
        if !self.is_user_process() {
            return None;
        }
        Some(UserProcessRef(
            self.0
                .map_addr(|addr| unsafe {
                    NonZero::new(addr.get() & !Self::PTR_TAG_MASK).unwrap_unchecked()
                })
                .cast::<UserProcessHeader>(),
        ))
    }

    pub fn as_builtin_process_ref(self) -> Option<BuiltinProcessRef> {
        if !self.is_builtin_process() {
            return None;
        }
        Some(BuiltinProcessRef(
            self.0
                .map_addr(|addr| unsafe {
                    NonZero::new(addr.get() & !Self::PTR_TAG_MASK).unwrap_unchecked()
                })
                .cast::<BuiltinProcessHeader>(),
        ))
    }

    pub fn as_any_process_ref(self) -> Option<AnyProcessRef> {
        AnyProcessRef::from_value(self)
    }

    pub fn as_symbol(self) -> Option<&'static Symbol> {
        if !self.is_symbol() {
            return None;
        }
        unsafe {
            Some(
                self.0
                    .map_addr(|addr| {
                        NonZero::new(addr.get() & !Self::PTR_TAG_MASK).unwrap_unchecked()
                    })
                    .cast::<Symbol>()
                    .as_ref(),
            )
        }
    }

    pub fn as_bool(self) -> Option<bool> {
        if self.0 == Self::FALSE.0 {
            return Some(false);
        }
        if self.0 == Self::TRUE.0 {
            return Some(true);
        }
        None
    }

    pub fn as_string_ref(self) -> Option<StringRef> {
        if !self.is_string() {
            return None;
        }
        Some(StringRef(
            self.0
                .map_addr(|addr| unsafe {
                    NonZero::new(addr.get() & !Self::PTR_TAG_MASK).unwrap_unchecked()
                })
                .cast::<StringHeader>(),
        ))
    }

    pub fn number_to_f64(self) -> Option<f64> {
        self.as_f64()
            .or_else(|| self.as_small_int().map(|n| n as f64))
            .or_else(|| {
                self.as_integer()
                    .map(|n| f64::rounding_from(n, RoundingMode::Nearest).0)
            })
    }

    pub fn int_to_integer(&self) -> Option<Integer> {
        self.as_int().map(|either| match either {
            Left(small) => Integer::from(small),
            Right(big) => big.clone(),
        })
    }

    pub fn int_to_i32(&self) -> Option<i32> {
        const ALWAYS_SMALL: bool = (Value::MIN_SMALL_INT as i128) <= i32::MIN as i128
            && Value::MAX_SMALL_INT as i128 >= i32::MAX as i128;
        if ALWAYS_SMALL {
            return self.as_small_int().and_then(|n| i32::try_from(n).ok());
        }
        self.as_int().and_then(|either| match either {
            Left(small) => i32::try_from(small).ok(),
            Right(big) => i32::try_from(big).ok(),
        })
    }

    pub fn int_to_u64(&self) -> Option<u64> {
        self.as_int().and_then(|either| match either {
            Left(small) => u64::try_from(small).ok(),
            Right(big) => u64::try_from(big).ok(),
        })
    }

    pub fn add(self, other: Value, gc: &mut GarbageCollector) -> Result<Value, TypeError> {
        if let (Some(n1), Some(n2)) = (self.as_int(), other.as_int()) {
            match (n1, n2) {
                (Left(n1), Left(n2)) => Ok(Self::alloc_from(n1 + n2, gc)),
                (Left(n1), Right(n2)) => Ok(Self::alloc_from(Integer::from(n1) + n2, gc)),
                (Right(n1), Left(n2)) => Ok(Self::alloc_from(n1 + Integer::from(n2), gc)),
                (Right(n1), Right(n2)) => Ok(Self::alloc_from(n1 + n2, gc)),
            }
        } else {
            let x1 = self.number_to_f64().ok_or(TypeError)?;
            let x2 = other.number_to_f64().ok_or(TypeError)?;
            Ok(Self::alloc_from(x1 + x2, gc))
        }
    }

    pub fn subtract(self, other: Value, gc: &mut GarbageCollector) -> Result<Value, TypeError> {
        if let (Some(n1), Some(n2)) = (self.as_int(), other.as_int()) {
            match (n1, n2) {
                (Left(n1), Left(n2)) => Ok(Self::alloc_from(n1 - n2, gc)),
                (Left(n1), Right(n2)) => Ok(Self::alloc_from(Integer::from(n1) - n2, gc)),
                (Right(n1), Left(n2)) => Ok(Self::alloc_from(n1 - Integer::from(n2), gc)),
                (Right(n1), Right(n2)) => Ok(Self::alloc_from(n1 - n2, gc)),
            }
        } else {
            let x1 = self.number_to_f64().ok_or(TypeError)?;
            let x2 = other.number_to_f64().ok_or(TypeError)?;
            Ok(Self::alloc_from(x1 - x2, gc))
        }
    }

    pub fn multiply(self, other: Value, gc: &mut GarbageCollector) -> Result<Value, TypeError> {
        if let (Some(n1), Some(n2)) = (self.as_int(), other.as_int()) {
            match (n1, n2) {
                (Left(n1), Left(n2)) => {
                    if let Some(product) = n1.checked_mul(n2) {
                        Ok(Self::alloc_from(product, gc))
                    } else {
                        Ok(Self::alloc_from(Integer::from(n1) * Integer::from(n2), gc))
                    }
                }
                (Left(n1), Right(n2)) => Ok(Self::alloc_from(Integer::from(n1) * n2, gc)),
                (Right(n1), Left(n2)) => Ok(Self::alloc_from(n1 * Integer::from(n2), gc)),
                (Right(n1), Right(n2)) => Ok(Self::alloc_from(n1 * n2, gc)),
            }
        } else {
            let x1 = self.number_to_f64().ok_or(TypeError)?;
            let x2 = other.number_to_f64().ok_or(TypeError)?;
            Ok(Self::alloc_from(x1 * x2, gc))
        }
    }

    pub fn divide(
        self,
        other: Value,
        gc: &mut GarbageCollector,
    ) -> Result<Option<Value>, TypeError> {
        if let (Some(n1), Some(n2)) = (self.as_int(), other.as_int()) {
            match (n1, n2) {
                (_, Left(0)) => Ok(None),
                (_, Right(zero)) if *zero == Integer::ZERO => Ok(None),
                (Left(n1), Left(n2)) => {
                    if let Some(quot) = n1.checked_div(n2) {
                        Ok(Some(Self::alloc_from(quot, gc)))
                    } else {
                        Ok(Some(Self::alloc_from(
                            Integer::from(n1) / Integer::from(n2),
                            gc,
                        )))
                    }
                }
                (Left(n1), Right(n2)) => Ok(Some(Self::alloc_from(Integer::from(n1) / n2, gc))),
                (Right(n1), Left(n2)) => Ok(Some(Self::alloc_from(n1 / Integer::from(n2), gc))),
                (Right(n1), Right(n2)) => Ok(Some(Self::alloc_from(n1 / n2, gc))),
            }
        } else {
            let x1 = self.number_to_f64().ok_or(TypeError)?;
            let x2 = other.number_to_f64().ok_or(TypeError)?;
            Ok(Some(Self::alloc_from(x1 / x2, gc)))
        }
    }

    pub fn remainder(
        self,
        other: Value,
        gc: &mut GarbageCollector,
    ) -> Result<Option<Value>, TypeError> {
        if let (Some(n1), Some(n2)) = (self.as_int(), other.as_int()) {
            match (n1, n2) {
                (_, Left(0)) => Ok(None),
                (_, Right(zero)) if *zero == Integer::ZERO => Ok(None),
                (Left(n1), Left(n2)) => {
                    if let Some(rem) = n1.checked_rem(n2) {
                        Ok(Some(Self::alloc_from(rem, gc)))
                    } else {
                        Ok(Some(Self::alloc_from(
                            Integer::from(n1) % Integer::from(n2),
                            gc,
                        )))
                    }
                }
                (Left(n1), Right(n2)) => Ok(Some(Self::alloc_from(Integer::from(n1) % n2, gc))),
                (Right(n1), Left(n2)) => Ok(Some(Self::alloc_from(n1 % Integer::from(n2), gc))),
                (Right(n1), Right(n2)) => Ok(Some(Self::alloc_from(n1 % n2, gc))),
            }
        } else {
            let x1 = self.number_to_f64().ok_or(TypeError)?;
            let x2 = other.number_to_f64().ok_or(TypeError)?;
            Ok(Some(Self::alloc_from(x1 % x2, gc)))
        }
    }

    pub fn negate(self, gc: &mut GarbageCollector) -> Result<Value, TypeError> {
        if let Some(n) = self.as_int() {
            match n {
                // Overflow can't happen, n won't be isize::MIN.
                Left(n) => Ok(Self::alloc_from(-n, gc)),
                Right(n) => Ok(Self::alloc_from(-n, gc)),
            }
        } else {
            let x = self.as_f64().ok_or(TypeError)?;
            Ok(Self::alloc_from(-x, gc))
        }
    }

    pub fn power(self, other: Self, gc: &mut GarbageCollector) -> Result<Self, PowerError> {
        if let Some(exp) = other.as_f64() {
            if let Some(base) = self.number_to_f64() {
                Ok(Self::alloc_from(base.powf(exp), gc))
            } else {
                Err(PowerError::TypeError)
            }
        } else if let Some(base) = self.as_f64() {
            if let Some(exp) = other.as_small_int()
                && let Ok(exp) = i32::try_from(exp)
            {
                Ok(Self::alloc_from(base.powi(exp), gc))
            } else if let Some(exp) = other.number_to_f64() {
                let mut result = base.powf(exp);
                if base < 0.0 {
                    let even = match other.as_int().unwrap() {
                        Left(n) => n % 2 == 0,
                        Right(n) => n.even(),
                    };
                    let sign = if even { 1.0 } else { -1.0 };
                    result = result.copysign(sign);
                }
                Ok(Self::alloc_from(result, gc))
            } else {
                Err(PowerError::TypeError)
            }
        } else if let Some(base) = self.int_to_integer() {
            if let Some(exp) = other.int_to_i32() {
                if exp < 0 {
                    if base == 1 {
                        Ok(self)
                    } else if base == -1 {
                        if exp % 2 == 0 {
                            Ok(Self::ONE)
                        } else {
                            Ok(self)
                        }
                    } else {
                        Err(PowerError::NegativeExponent)
                    }
                } else {
                    Ok(Self::alloc_from(base.pow(exp as u64), gc))
                }
            } else if let Some(exp) = other.as_int() {
                if base == 0 || base == 1 {
                    Ok(self)
                } else if base == -1 {
                    let even = match exp {
                        Left(exp) => exp % 2 == 0,
                        Right(exp) => exp.even(),
                    };
                    if even { Ok(Self::ONE) } else { Ok(self) }
                } else {
                    let negative = match exp {
                        Left(exp) => exp < 0,
                        Right(exp) => *exp < 0,
                    };
                    if negative {
                        Err(PowerError::NegativeExponent)
                    } else {
                        Err(PowerError::Overflow)
                    }
                }
            } else {
                Err(PowerError::TypeError)
            }
        } else {
            Err(PowerError::TypeError)
        }
    }

    pub fn compare(self, other: Self) -> Result<Option<Ordering>, TypeError> {
        // For compare_small_to_f64, n1 must not be isize::MIN or isize::MAX.
        fn compare_small_to_f64(n1: isize, x2: f64) -> Option<Ordering> {
            if x2.is_nan() {
                return None;
            }
            let ord = n1.cmp(&(x2 as isize));
            if ord.is_ne() {
                return Some(ord);
            }
            0.0.partial_cmp(&x2.fract())
        }

        if let Some(n1) = self.as_int() {
            if let Some(n2) = other.as_int() {
                match (n1, n2) {
                    (Left(n1), Left(n2)) => Ok(n1.partial_cmp(&n2)),
                    (Left(n1), Right(n2)) => Ok(n1.partial_cmp(n2)),
                    (Right(n1), Left(n2)) => Ok(n1.partial_cmp(&n2)),
                    (Right(n1), Right(n2)) => Ok(n1.partial_cmp(n2)),
                }
            } else {
                let x2 = other.as_f64().ok_or(TypeError)?;
                match n1 {
                    Left(n1) => Ok(compare_small_to_f64(n1, x2)),
                    Right(n1) => Ok(n1.partial_cmp(&x2)),
                }
            }
        } else if let Some(x1) = self.as_f64() {
            if let Some(n2) = other.as_int() {
                match n2 {
                    Left(n2) => Ok(compare_small_to_f64(n2, x1).map(Ordering::reverse)),
                    Right(n2) => Ok(x1.partial_cmp(n2)),
                }
            } else {
                let x2 = other.as_f64().ok_or(TypeError)?;
                Ok(x1.partial_cmp(&x2))
            }
        } else if let Some(s1) = self.as_string_ref() {
            let s2 = other.as_string_ref().ok_or(TypeError)?;
            Ok(s1.bytes().partial_cmp(s2.bytes()))
        } else {
            Err(TypeError)
        }
    }

    pub fn write_to<W: io::Write>(self, output: &mut W) -> io::Result<()> {
        if let Some(s) = self.as_string_ref() {
            output.write_all(s.bytes())
        } else if let Some(n) = self.as_small_int() {
            write!(output, "{}", n)
        } else if let Some(n) = self.as_big_int_ref() {
            write!(output, "{}", unsafe { n.value().as_ref().unwrap() })
        } else if let Some(f) = self.as_float_ref() {
            write!(output, "{}", f.to_str(&mut ryu::Buffer::new()))
        } else if let Some(sym) = self.as_symbol() {
            write!(output, "{}", sym.name())
        } else if let Some(proc) = self.as_user_process_ref() {
            write!(output, "<process at {:p}>", proc.0)
        } else if let Some(proc) = self.as_builtin_process_ref() {
            write!(output, "<{} process at {:p}>", proc.type_name(), proc.0)
        } else {
            todo!()
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.as_string_ref() {
            write!(f, "{:?}", String::from_utf8_lossy(s.bytes()))
        } else if let Some(n) = self.as_small_int() {
            write!(f, "{}", n)
        } else if let Some(n) = self.as_big_int_ref() {
            write!(f, "{}", unsafe { n.value().as_ref().unwrap() })
        } else if let Some(fr) = self.as_float_ref() {
            write!(f, "{}", fr.to_str(&mut ryu::Buffer::new()))
        } else if let Some(sym) = self.as_symbol() {
            write!(f, ".{}", sym.name())
        } else if let Some(proc) = self.as_user_process_ref() {
            write!(f, "<process at {:p}>", proc.0)
        } else if let Some(proc) = self.as_builtin_process_ref() {
            write!(f, "<{} process at {:p}>", proc.type_name(), proc.0)
        } else {
            write!(
                f,
                "Value(0x{:0width$x})",
                self.as_addr(),
                width = usize::BITS as usize / 4
            )
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        fn small_eq_f64(small: isize, float: f64) -> bool {
            if float.fract() != 0.0 {
                return false;
            }
            const TOO_BIG: f64 = (isize::MAX as u128 + 1) as f64;
            if float.abs() >= TOO_BIG {
                return false;
            }
            small == float as isize
        }
        if let (Some(f1), Some(f2)) = (self.as_f64(), other.as_f64()) {
            return f1 == f2;
        }
        if self.as_addr() == other.as_addr() {
            return true;
        }
        match (self.tag(), other.tag()) {
            (Self::SMALL_INT_TAG, Self::FLOAT_TAG) => {
                small_eq_f64(self.as_small_int().unwrap(), other.as_f64().unwrap())
            }
            (Self::FLOAT_TAG, Self::SMALL_INT_TAG) => {
                small_eq_f64(other.as_small_int().unwrap(), self.as_f64().unwrap())
            }
            (
                Self::BUILTIN_PROCESS_TAG
                | Self::USER_PROCESS_TAG
                | Self::SMALL_INT_TAG
                | Self::SYMBOL_TAG,
                _,
            )
            | (
                _,
                Self::BUILTIN_PROCESS_TAG
                | Self::USER_PROCESS_TAG
                | Self::SMALL_INT_TAG
                | Self::SYMBOL_TAG,
            ) => false,
            (Self::BIG_INT_TAG, Self::BIG_INT_TAG) => unsafe {
                *self.as_big_int_ref().unwrap().value() == *other.as_big_int_ref().unwrap().value()
            },
            (Self::STRING_TAG, Self::STRING_TAG) => {
                self.as_string_ref().unwrap().bytes() == other.as_string_ref().unwrap().bytes()
            }
            (Self::FLOAT_TAG, Self::BIG_INT_TAG) => {
                self.as_f64().unwrap() == *other.as_integer().unwrap()
            }
            (Self::BIG_INT_TAG, Self::FLOAT_TAG) => {
                *self.as_integer().unwrap() == other.as_f64().unwrap()
            }
            (Self::CAPTURE_TAG, _) | (_, Self::CAPTURE_TAG) => todo!(),
            (Self::STRING_TAG, _) | (_, Self::STRING_TAG) => false,
            (Self::FLOAT_TAG, Self::FLOAT_TAG) | (8.., _) | (_, 8..) => unreachable!(),
        }
    }
}

impl From<BuiltinProcessRef> for Value {
    fn from(proc: BuiltinProcessRef) -> Self {
        Self(
            proc.0
                .map_addr(|addr| (addr | Self::BUILTIN_PROCESS_TAG))
                .cast::<u8>(),
        )
    }
}

impl From<UserProcessRef> for Value {
    fn from(proc: UserProcessRef) -> Self {
        Self(
            proc.0
                .map_addr(|addr| (addr | Self::USER_PROCESS_TAG))
                .cast::<u8>(),
        )
    }
}

impl From<AnyProcessRef> for Value {
    fn from(proc: AnyProcessRef) -> Self {
        proc.0
    }
}

impl From<BigIntRef> for Value {
    fn from(int_ref: BigIntRef) -> Self {
        Self(
            int_ref
                .0
                .map_addr(|addr| (addr | Self::BIG_INT_TAG))
                .cast::<u8>(),
        )
    }
}

impl From<FloatRef> for Value {
    fn from(f: FloatRef) -> Self {
        Self(f.0.map_addr(|addr| addr | Self::FLOAT_TAG).cast::<u8>())
    }
}

impl From<&'static Symbol> for Value {
    fn from(sym: &'static Symbol) -> Self {
        Self(
            NonNull::from(sym)
                .map_addr(|addr| (addr | Self::SYMBOL_TAG))
                .cast::<u8>(),
        )
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        if b { Self::TRUE } else { Self::FALSE }
    }
}

impl From<StringRef> for Value {
    fn from(s: StringRef) -> Self {
        Self(s.0.map_addr(|addr| (addr | Self::STRING_TAG)).cast::<u8>())
    }
}

impl From<CaptureRef> for Value {
    fn from(capture: CaptureRef) -> Self {
        Self(
            capture
                .0
                .map_addr(|addr| (addr | Self::CAPTURE_TAG))
                .cast::<u8>(),
        )
    }
}
pub trait AllocIntoValue {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value;
}

impl AllocIntoValue for isize {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        if let Some(result) = Value::from_small_int(self) {
            result
        } else {
            Value::from(BigIntRef::new_unchecked(Integer::from(self), gc))
        }
    }
}

impl AllocIntoValue for usize {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        if let Ok(n) = isize::try_from(self)
            && let Some(result) = Value::from_small_int(n)
        {
            result
        } else {
            Value::from(BigIntRef::new_unchecked(Integer::from(self), gc))
        }
    }
}

impl AllocIntoValue for i64 {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        if let Ok(n) = isize::try_from(self) {
            Value::alloc_from(n, gc)
        } else {
            Value::from(BigIntRef::new_unchecked(Integer::from(self), gc))
        }
    }
}

impl AllocIntoValue for u64 {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        if let Ok(n) = isize::try_from(self) {
            Value::alloc_from(n, gc)
        } else {
            Value::from(BigIntRef::new_unchecked(Integer::from(self), gc))
        }
    }
}

impl AllocIntoValue for Integer {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        if let Ok(n) = isize::try_from(&self)
            && let Some(result) = Value::from_small_int(n)
        {
            result
        } else {
            Value::from(BigIntRef::new_unchecked(self, gc))
        }
    }
}

impl AllocIntoValue for f64 {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        Value::from(FloatRef::new(self, gc))
    }
}

impl AllocIntoValue for &'_ [u8] {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        Value::from(StringRef::new(self, gc))
    }
}

impl AllocIntoValue for &'_ str {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        Value::alloc_from(self.as_bytes(), gc)
    }
}

impl AllocIntoValue for String {
    fn alloc_into_value(self, gc: &mut GarbageCollector) -> Value {
        Value::alloc_from(&self[..], gc)
    }
}

#[cfg(test)]
mod test {
    use std::isize;

    use super::*;

    #[test]
    fn small_int_test() {
        let val1 = Value::from_small_int(123).unwrap();
        assert_eq!(val1.as_small_int().unwrap(), 123);
        let val2 = Value::from_small_int(-123).unwrap();
        assert_eq!(val2.as_small_int().unwrap(), -123);
        assert!(Value::from_small_int(isize::MIN).is_none());
        assert!(Value::from_small_int(isize::MAX).is_none());
    }
}
